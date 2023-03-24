use super::lua_request::LuaRequest;
use crate::error::Error as WebError;
use crate::lua::response::HiveResponse;
use crate::lua::router::HiveRouter;
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
    handler: Option<Arc<LuaRegistryKey>>,
    exception: Arc<LuaRegistryKey>,
    router: Option<Arc<HiveRouter>>,
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
        let _router = self.router.clone();
        log::info!(
            "Request -- remote address: {}, method: {}, uri: {}",
            self.remote_addr,
            method,
            path
        );

        Box::pin(async move {
            let handler: Option<LuaFunction> = if let Some(_handler) = handler {
                #[cfg(feature = "add_entrance")]
                {
                    Some(lua.registry_value(&_handler)?)
                }
                #[cfg(not(feature = "add_entrance"))]
                {
                    None
                }
            } else {
                None
            };

            let exception: LuaFunction = lua.registry_value(&exception)?;

            #[cfg(not(feature = "lua_hotfix"))]
            {
                let lua_req = lua.create_userdata(lua_req)?;
                if let Some(router) = _router {
                    match router
                        .execute(method, path, lua_req, exception.clone(), handler)
                        .await
                    {
                        Ok(lua_resp) => match lua_resp {
                            LuaValue::UserData(v) => {
                                let resp = v.take::<HiveResponse<Body>>()?;
                                Ok(resp.0)
                            }
                            _ => {
                                let body = serde_json::to_vec(&lua_resp)?;
                                let resp = Response::new(Body::from(body));
                                Ok(resp)
                            }
                        },
                        Err(err) => {
                            println!("{err:?}");

                            // let (code, message) = return_err_info(err);
                            log::error!("{}", err.message);
                            let resp = exception
                                .call_async::<_, LuaValue>((err.code, err.message))
                                .await?;
                            match resp {
                                LuaValue::UserData(v) => {
                                    let resp = v.take::<HiveResponse<Body>>()?;
                                    Ok(resp.0)
                                }
                                _ => {
                                    let body = serde_json::to_vec(&resp)?;
                                    let resp = Response::new(Body::from(body));
                                    Ok(resp)
                                }
                            }
                        }
                    }
                } else {
                    Ok(Response::new(Body::empty()))
                }
            }
            #[cfg(feature = "lua_hotfix")]
            if let Some(handler) = handler {
                match handler.call_async((method, path, lua_req)).await {
                    Ok(lua_resp) => match lua_resp {
                        LuaValue::UserData(v) => {
                            let resp = v.take::<HiveResponse<Body>>()?;
                            Ok(resp.0)
                        }
                        _ => {
                            let body = serde_json::to_vec(&lua_resp)?;
                            let resp = Response::new(Body::from(body));
                            Ok(resp)
                        }
                    },
                    Err(err) => {
                        println!("{err:?}");

                        let (code, message) = return_err_info(err);
                        log::error!("{}", message);
                        let resp = exception.call_async::<_, LuaValue>((code, message)).await?;
                        match resp {
                            LuaValue::UserData(v) => {
                                let resp = v.take::<HiveResponse<Body>>()?;
                                Ok(resp.0)
                            }
                            _ => {
                                let body = serde_json::to_vec(&resp)?;
                                let resp = Response::new(Body::from(body));
                                Ok(resp)
                            }
                        }
                    }
                }
            } else {
                Ok(Response::new(Body::empty()))
            }
        })
    }
}

#[cfg(feature = "lua_hotfix")]
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
    pub handler: Option<Arc<LuaRegistryKey>>,
    pub exception: Arc<LuaRegistryKey>,
    pub router: Option<Arc<HiveRouter>>,
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
        let router = self.router.clone();

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
                    router,
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
