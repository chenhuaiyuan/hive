mod crypto;
mod error;
mod file;
mod json;
mod jwt_simple;
mod lua_http;
mod mysql;
mod nanoid;

use crate::crypto::LuaCrypto;
use crate::error::{create_error, Error as WebError, Result as WebResult};
use crate::file::File;
use crate::json::create_table_to_json_string;
use crate::jwt_simple::HS256;
use crate::lua_http::Http;
use crate::mysql::MysqlPool;
use crate::nanoid::create_nanoid;
use clap::Parser;
use futures_util::Future;
use http::{header, HeaderMap, Method};
use http::{header::HeaderValue, header::CONTENT_TYPE};
use hyper::service::Service;
use hyper::{server::conn::AddrStream, Body, Request, Response, Server};
use mlua::prelude::*;
use multer::Multipart;
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
                    return Ok(params_table);
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
        _methods.add_method("remoteAddr", |_, this, ()| Ok((this.1).to_string()));
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
        _methods.add_async_function("form", |lua, this: LuaAnyUserData| async move {
            let form_data = lua.create_table()?;
            let this = this.take::<Self>()?;
            if !has_content_type(this.0.headers(), &mime::MULTIPART_FORM_DATA) {
                return Ok(form_data);
            }
            let boundary = this
                .0
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|ct| ct.to_str().ok())
                .and_then(|ct| multer::parse_boundary(ct).ok());

            let mut multipart = Multipart::new(this.0.into_body(), boundary.unwrap());

            while let Some(mut field) = multipart.next_field().await.to_lua_err()? {
                let name = field.name().map(|v| v.to_string());

                let file_name = field.file_name().map(|v| v.to_string());

                let content_type = field.content_type().map(|v| v.to_string());

                // println!(
                //     "Name: {:?}, FileName: {:?}, Content-Type: {:?}",
                //     name, file_name, content_type
                // );

                let mut field_data: Vec<u8> = Vec::new();
                // let mut field_bytes_len = 0;
                while let Some(field_chunk) = field.chunk().await.to_lua_err()? {
                    // Do something with field chunk.
                    // field_bytes_len += field_chunk.len();
                    field_data.append(&mut field_chunk.to_vec())
                    // println!("{:?}", field_chunk);
                }

                if let Some(file_name) = file_name.clone() {
                    let field_name = name.clone().unwrap_or_else(|| "default".to_string());
                    let content_type = content_type
                        .clone()
                        .unwrap_or_else(|| "multipart/form-data".to_string());
                    let file = File::new(field_name.clone(), file_name, content_type, field_data);
                    form_data.set(field_name, file)?;
                } else if let Some(field_name) = name.clone() {
                    let data = lua.create_string(&String::from_utf8(field_data).to_lua_err()?)?;
                    form_data.set(field_name, data)?;
                }
            }

            Ok(form_data)
        });
    }
}

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
                        .get::<_, Option<LuaValue>>("body")
                        .to_lua_err()?
                        .map(|b| match b {
                            LuaValue::String(v) => Body::from(v.as_bytes().to_vec()),
                            LuaValue::UserData(v) => {
                                let this = v.take::<File>().unwrap();
                                Body::from(this.content)
                            }
                            _ => Body::from(""),
                        })
                        .unwrap_or_else(Body::empty);

                    Ok(resp.body(body).unwrap())
                }
                Err(err) => {
                    println!("{:?}", err);
                    let (code, message) = return_err_info(err);
                    Ok(Response::builder()
                        .status(200)
                        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                        .body(Body::from(format!(
                            r#"{{"code": {}, "message": "{}", "data": ""}}"#,
                            code, message
                        )))
                        .unwrap())
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
            (r[0].parse::<u16>().unwrap_or(500u16), r[1].to_string())
        }
        _ => (4017, err.to_string()),
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
    let lua;
    unsafe {
        lua = Rc::new(Lua::unsafe_new());
    }
    let lua_clone = lua.clone();

    let globals = lua_clone.globals();

    globals.set("MysqlPool", lua.create_proxy::<MysqlPool>()?)?;
    globals.set("tableToJsonStr", create_table_to_json_string(&lua)?)?;
    globals.set("nanoid", create_nanoid(&lua)?)?;
    globals.set("JWTHS256", lua.create_proxy::<HS256>()?)?;
    globals.set("File", lua.create_proxy::<File>()?)?;
    globals.set("Http", lua.create_proxy::<Http>()?)?;
    globals.set("Crypto", lua.create_proxy::<LuaCrypto>()?)?;
    globals.set("WebError", create_error(&lua)?)?;

    // let env = lua.create_table()?;
    // env.set("crypto", LuaCrypto)?;

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
