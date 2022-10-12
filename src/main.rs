mod error;
mod json;
mod jwt_simple;
mod mysql;
mod nanoid;

use crate::error::{Error as WebError, Result as WebResult};
use crate::json::create_table_to_json_string;
use crate::jwt_simple::HS256;
use crate::mysql::MysqlPool;
use crate::nanoid::create_nanoid;
use axum_core::response::IntoResponse;
use clap::Parser;
use futures_util::Future;
use http::{header, HeaderMap, Method};
use http::{header::HeaderValue, header::CONTENT_TYPE};
use hyper::service::Service;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use mlua::prelude::*;
use std::net::SocketAddr;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;

fn has_content_type(headers: &HeaderMap, expected_content_type: &mime::Mime) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    content_type.starts_with(expected_content_type.as_ref())
}

struct LuaRequest(Request<Body>, SocketAddr);

impl LuaUserData for LuaRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_async_function("params", |_lua, this: LuaAnyUserData| async move {
            let params_table = _lua.create_table()?;
            let this = this.take::<Self>()?;
            if this.0.method() == Method::GET {
                let query = this.0.uri().query().unwrap_or_default();
                let value = serde_urlencoded::from_str::<Vec<(String, String)>>(query)
                    .map_err(WebError::parse_params)
                    .to_lua_err()?;
                for (key, val) in value {
                    params_table.set(key, val)?;
                }
                Ok(params_table)
            } else {
                if !has_content_type(this.0.headers(), &mime::APPLICATION_WWW_FORM_URLENCODED) {
                    return Err(WebError::invalid_form_content_type().to_lua_err());
                }
                let bytes = hyper::body::to_bytes(this.0).await.to_lua_err()?;
                let value = serde_urlencoded::from_bytes::<Vec<(String, String)>>(&bytes)
                    .map_err(WebError::parse_params)
                    .to_lua_err()?;
                for (key, val) in value {
                    params_table.set(key, val)?;
                }
                Ok(params_table)
            }
        });
        _methods.add_method("remote_addr", |_, this, ()| Ok((this.1).to_string()));
        _methods.add_method("headers", |lua, this, ()| {
            let headers = lua.create_table()?;
            let headers_raw = this.0.headers();
            for (key, val) in headers_raw {
                let key = key.as_str().to_string();
                let val = val.to_str().to_lua_err()?.to_string();
                headers.set(key, val)?;
            }
            Ok(headers)
        });
    }
}

// async fn run_server(handler: LuaFunction<'static>, lua: &Lua) -> WebResult<()> {
//     let make_svc = make_service_fn(|socket: &AddrStream| {
//         let remote_addr = socket.remote_addr();
//         let handler = handler.clone();
//         async move {
//             Ok::<_, WebError>(service_fn(move |req: Request<Body>| {
//                 let handler = handler.clone();
//                 let method = req.method().as_str().to_string();
//                 let path = req.uri().path().to_string();
//                 async move {
//                     let lua_req = LuaRequest(req, remote_addr);
//                     let lua_resp: LuaTable = handler.call_async((method, path, lua_req)).await?;
//                     let status = lua_resp
//                         .get::<_, Option<u16>>("status")
//                         .to_lua_err()?
//                         .unwrap_or(200);
//                     let mut resp = Response::builder().status(status);

//                     if let Some(headers) = lua_resp
//                         .get::<_, Option<LuaTable>>("headers")
//                         .to_lua_err()?
//                     {
//                         for pair in headers.pairs::<String, LuaString>() {
//                             let (h, v) = pair.to_lua_err()?;
//                             resp = resp.header(&h, v.as_bytes());
//                         }
//                     }

//                     let body = lua_resp
//                         .get::<_, Option<LuaString>>("body")
//                         .to_lua_err()?
//                         .map(|b| {
//                             // let b = serde_json::to_string(&b).unwrap();
//                             Body::from(b.as_bytes().to_vec())
//                         })
//                         .unwrap_or_else(Body::empty);

//                     Ok::<_, WebError>(resp.body(Body::from(body)).unwrap())
//                 }
//             }))
//         }
//     });

//     let localhost: String = lua.globals().get("LOCALHOST")?;

//     // let addr = ([127, 0, 0, 1], 3000).into();
//     let addr = localhost.parse()?;
//     let server = Server::bind(&addr).executor(LocalExec).serve(make_svc);

//     println!("Listening on http://{}", addr);

//     let local = tokio::task::LocalSet::new();
//     local.run_until(server).await?;
//     Ok(())
// }

struct Svc(Rc<Lua>, SocketAddr);

impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = WebError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let lua = self.0.clone();
        let method = req.method().as_str().to_string();
        let path = req.uri().path().to_string();
        let lua_req = LuaRequest(req, self.1);

        Box::pin(async move {
            let handler: LuaFunction = lua.named_registry_value("http_handler")?;

            match handler
                .call_async::<_, LuaTable>((method, path, lua_req))
                .await
            {
                Ok(lua_resp) => {
                    let status = lua_resp
                        .get::<_, Option<u16>>("status")
                        .to_lua_err()?
                        .unwrap_or(200);
                    let mut resp = Response::builder().status(status);

                    if let Some(headers) = lua_resp
                        .get::<_, Option<LuaTable>>("headers")
                        .to_lua_err()?
                    {
                        for pair in headers.pairs::<String, LuaString>() {
                            let (h, v) = pair.to_lua_err()?;
                            resp = resp.header(&h, v.as_bytes());
                        }
                    }

                    let body = lua_resp
                        .get::<_, Option<LuaString>>("body")
                        .to_lua_err()?
                        .map(|b| Body::from(b.as_bytes().to_vec()))
                        .unwrap_or_else(Body::empty);

                    Ok(resp.body(body).unwrap())
                }
                Err(err) => {
                    println!("{:?}", err);
                    Ok(Response::builder()
                        .status(500)
                        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                        .body(Body::from(
                            r#"{"code": 500, "message": "Call failed", "data": ""}"#,
                        ))
                        .unwrap())
                }
            }
        })
    }
}

struct MakeSvc(Rc<Lua>);

impl Service<&AddrStream> for MakeSvc {
    type Response = Svc;
    type Error = WebError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, stream: &AddrStream) -> Self::Future {
        let lua = self.0.clone();
        let remote_addr = stream.remote_addr();
        Box::pin(async move { Ok(Svc(lua, remote_addr)) })
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> WebResult<()> {
    let args = Args::parse();

    // let lua = Lua::new().into_static();
    let lua = Rc::new(Lua::new());
    let lua_clone = lua.clone();

    let globals = lua_clone.globals();

    globals.set("MysqlPool", lua.create_proxy::<MysqlPool>()?)?;
    globals.set("tableToJsonStr", create_table_to_json_string(&lua)?)?;
    globals.set("nanoid", create_nanoid(&lua)?)?;
    globals.set("JWTHS256", lua.create_proxy::<HS256>()?)?;

    let file = tokio::fs::read_to_string(args.file)
        .await
        .expect("read file failed");

    let handler: LuaFunction = lua.load(&file).eval()?;
    let localhost: String = lua.globals().get("LOCALHOST")?;
    let addr = localhost.parse()?;
    println!("Listening on http://{}", addr);
    lua.set_named_registry_value("http_handler", handler)?;
    let server = Server::bind(&addr).executor(LocalExec).serve(MakeSvc(lua));

    let local = tokio::task::LocalSet::new();
    local.run_until(server).await?;

    // run_server(handler, lua).await?;
    // unsafe { Lua::from_static(lua) };
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
where
    F: std::future::Future + 'static, // not requiring `Send`
{
    fn execute(&self, fut: F) {
        tokio::task::spawn_local(fut);
    }
}
