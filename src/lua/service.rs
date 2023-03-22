use super::lua_request::LuaRequest;
use crate::error::Error as WebError;
use futures_util::Future;

#[cfg(feature = "h2")]
use crate::LocalExec;
#[cfg(feature = "h2")]
use http::header::UPGRADE;
#[cfg(feature = "h2")]
use http::StatusCode;
// use http::Version;
use hyper::{server::conn::AddrStream, service::Service, Body, Request, Response};
use mlua::prelude::*;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

// pub struct Svc(Arc<Lua>, SocketAddr);
pub struct Svc {
    lua: Arc<Lua>,
    remote_addr: SocketAddr,
    handler: Arc<LuaRegistryKey>,
    exception: Arc<LuaRegistryKey>,
}

impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = WebError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let lua: Arc<Lua> = self.lua.clone();
        let method: String = req.method().as_str().to_string();
        let path: String = req.uri().path().to_string();
        let lua_req: LuaRequest = LuaRequest::new(req, self.remote_addr);
        let handler = self.handler.clone();
        let exception = self.exception.clone();
        log::info!(
            "Request -- remote address: {}, method: {}, uri: {}",
            self.remote_addr,
            method,
            path
        );

        Box::pin(async move {
            // let handler: LuaFunction = lua.named_registry_value("http_handler")?;
            let handler: LuaFunction = lua.registry_value(&handler)?;

            match handler
                .call_async::<_, LuaValue>((method, path, lua_req))
                .await
            {
                Ok(lua_resp) => match lua_resp {
                    LuaValue::Integer(v) => Ok(Response::new(Body::from(v.to_string()))),
                    LuaValue::Number(v) => Ok(Response::new(Body::from(v.to_string()))),
                    LuaValue::String(v) => Ok(Response::new(Body::from(v.to_str()?.to_string()))),
                    LuaValue::Table(v) => {
                        let status: u16 = v
                            .get::<_, Option<u16>>("status")
                            .to_lua_err()?
                            .unwrap_or(200);
                        let mut resp: http::response::Builder = Response::builder().status(status);

                        // let version: Option<String> =
                        //     v.get::<_, Option<String>>("version").to_lua_err()?;
                        // if let Some(ver) = version {
                        //     if ver == "HTTP/0.9" {
                        //         resp = resp.version(Version::HTTP_09);
                        //     } else if ver == "HTTP/1.0" {
                        //         resp = resp.version(Version::HTTP_10);
                        //     } else if ver == "HTTP/1.1" {
                        //         resp = resp.version(Version::HTTP_11);
                        //     } else if ver == "HTTP/2.0" {
                        //         resp = resp.version(Version::HTTP_2);
                        //     } else if ver == "HTTP/3.0" {
                        //         resp = resp.version(Version::HTTP_3);
                        //     }
                        // }

                        if let Some(headers) =
                            v.get::<_, Option<LuaTable>>("headers").to_lua_err()?
                        {
                            for pair in headers.pairs::<String, LuaString>() {
                                let (h, v) = pair.to_lua_err()?;
                                resp = resp.header(&h, v.as_bytes());
                            }
                        }

                        let body = v
                            .get::<_, Option<LuaString>>("body")
                            .to_lua_err()?
                            .map(|b| Body::from(b.as_bytes().to_vec()))
                            .unwrap_or_else(Body::empty);

                        Ok(resp.body(body).unwrap())
                    }
                    _ => Ok(Response::new(Body::empty())),
                },
                Err(err) => {
                    println!("{err:?}");
                    // let exception: LuaFunction = lua.named_registry_value("exception")?;
                    let exception: LuaFunction = lua.registry_value(&exception)?;
                    let (code, message) = return_err_info(err);
                    log::error!("{}", message);
                    let resp = exception.call_async::<_, LuaValue>((code, message)).await?;
                    match resp {
                        LuaValue::Integer(v) => Ok(Response::new(Body::from(v.to_string()))),
                        LuaValue::Number(v) => Ok(Response::new(Body::from(v.to_string()))),
                        LuaValue::String(v) => {
                            Ok(Response::new(Body::from(v.to_str()?.to_string())))
                        }
                        LuaValue::Table(v) => {
                            let status = v
                                .get::<_, Option<u16>>("status")
                                .to_lua_err()?
                                .unwrap_or(200);
                            let mut resp = Response::builder().status(status);

                            if let Some(headers) =
                                v.get::<_, Option<LuaTable>>("headers").to_lua_err()?
                            {
                                for pair in headers.pairs::<String, LuaString>() {
                                    let (h, v) = pair.to_lua_err()?;
                                    resp = resp.header(&h, v.as_bytes());
                                }
                            }

                            let body: Body = v
                                .get::<_, Option<LuaString>>("body")
                                .to_lua_err()?
                                .map(|b| Body::from(b.as_bytes().to_vec()))
                                .unwrap_or_else(Body::empty);

                            Ok(resp.body(body).unwrap())
                        }
                        _ => Ok(Response::new(Body::empty())),
                    }
                    // Ok(Response::builder()
                    //     .status(200)
                    //     .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                    //     .body(Body::from(format!(
                    //         r#"{{"code": {code}, "message": "{message}", "data": ""}}"#
                    //     )))
                    //     .unwrap())
                }
            }
        })
    }
}

fn return_err_info(err: LuaError) -> (u16, String) {
    match err {
        LuaError::SyntaxError {
            message,
            incomplete_input: _,
        } => (4005, message),
        LuaError::RuntimeError(v) => (4006, v),
        LuaError::MemoryError(v) => (4007, v),
        LuaError::SafetyError(v) => (4009, v),
        LuaError::ToLuaConversionError {
            from: _,
            to: _,
            message,
        } => (
            4010,
            message.unwrap_or_else(|| "To Lua Conversion Error".to_string()),
        ),
        LuaError::FromLuaConversionError {
            from: _,
            to: _,
            message,
        } => (
            4011,
            message.unwrap_or_else(|| "From Lua Conversion Error".to_string()),
        ),
        LuaError::MetaMethodRestricted(v) => (4012, v),
        LuaError::MetaMethodTypeError {
            method: _,
            type_name: _,
            message,
        } => (
            4013,
            message.unwrap_or_else(|| "Meta Method Type Error".to_string()),
        ),
        LuaError::CallbackError {
            traceback: _,
            cause,
        } => {
            let err = cause.as_ref();
            return_err_info(err.clone())
        }
        LuaError::SerializeError(v) => (4015, v),
        LuaError::DeserializeError(v) => (4016, v),
        LuaError::ExternalError(v) => {
            let s = v.as_ref().to_string();
            let r: Vec<&str> = s.split(',').collect();
            if r.len() >= 2 {
                (r[0].parse::<u16>().unwrap_or(500u16), r[1].to_string())
            } else {
                (500, s)
            }
        }
        _ => (4017, err.to_string()),
    }
}

pub struct MakeSvc {
    pub lua: Arc<Lua>,
    pub handler: Arc<LuaRegistryKey>,
    pub exception: Arc<LuaRegistryKey>,
}

impl Service<&AddrStream> for MakeSvc {
    #[cfg(not(feature = "h2"))]
    type Response = Svc;

    #[cfg(feature = "h2")]
    type Response = H2Svc;

    type Error = WebError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, stream: &AddrStream) -> Self::Future {
        let lua = self.lua.clone();
        let handler = self.handler.clone();
        let exception = self.exception.clone();
        let remote_addr = stream.remote_addr();

        #[cfg(feature = "h2")]
        {
            Box::pin(async move { Ok(H2Svc(lua, remote_addr)) })
        }
        #[cfg(not(feature = "h2"))]
        {
            Box::pin(async move {
                Ok(Svc {
                    lua,
                    remote_addr,
                    handler,
                    exception,
                })
            })
        }
    }
}

#[cfg(feature = "h2")]
pub struct H2Svc(pub Arc<Lua>, SocketAddr);

#[cfg(feature = "h2")]
impl Service<Request<Body>> for H2Svc {
    type Response = Response<Body>;
    type Error = WebError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let lua = self.0.clone();
        let remote_addr = self.1.clone();
        let mut res = Response::new(Body::empty());
        if !req.headers().contains_key(UPGRADE) {
            *res.status_mut() = StatusCode::BAD_REQUEST;
            return Box::pin(async move { Ok(res) });
        }
        Box::pin(async move {
            let conn = hyper::upgrade::on(req).await?;
            let http = hyper::server::conn::Http::new();
            http.with_executor(LocalExec)
                .serve_connection(conn, Svc(lua, remote_addr))
                .await?;

            *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
            res.headers_mut()
                .insert(UPGRADE, http::HeaderValue::from_static("h2c"));
            Ok(res)
            // Ok(Response::builder()
            //     .status(StatusCode::SWITCHING_PROTOCOLS)
            //     .header(UPGRADE, "h2c")
            //     .body(Body::empty())?)
        })
    }
}
