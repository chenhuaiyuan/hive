mod error;
#[cfg(any(
    feature = "alipay",
    feature = "crypto",
    feature = "file",
    feature = "json",
    feature = "jwt_simple",
    feature = "http",
    feature = "mysql",
    feature = "nanoid"
))]
mod utils;

use crate::error::{create_error, Error as WebError, Result as WebResult};
use crate::utils::{
    alipay::create_alipay, crypto::LuaCrypto, file::File, json::create_table_to_json_string,
    jwt_simple::HS256, lua_http::Http, mysql::MysqlPool, nanoid::create_nanoid,
};
use clap::Parser;
use fast_log::{
    config::Config,
    consts::LogSize,
    plugin::{file_split::RollingType, packer::ZipPacker},
};
use futures_util::Future;
use http::{header, HeaderMap, Method};
use http::{header::HeaderValue, header::CONTENT_TYPE};
use hyper::service::Service;
use hyper::{server::conn::AddrStream, Body, Request, Response, Server};
use mlua::prelude::*;
use multer::Multipart;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
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
        _methods.add_async_function("params", |lua, this: LuaAnyUserData| async move {
            let params_table = lua.create_table()?;
            let this = this.take::<Self>()?;
            if this.0.method() == Method::GET {
                let query = this.0.uri().query().unwrap_or_default();
                let value = serde_urlencoded::from_str::<Vec<(String, String)>>(query)
                    .map_err(WebError::parse_params)
                    .to_lua_err()?;

                let mut param: HashMap<String, LuaValue> = HashMap::new();
                for (key, val) in value {
                    let offset = key.find('[');
                    if let Some(o) = offset {
                        let k = key.get(0..o);
                        if let Some(k) = k {
                            let offset = key.find('.');
                            // 表明是对象
                            if let Some(i) = offset {
                                let right_offset = key.find(']');
                                let index;
                                if let Some(r) = right_offset {
                                    let a = key.get((o + 1)..r).unwrap_or("1");
                                    let temp = a.parse::<i32>().to_lua_err()?;
                                    index = temp + 1;
                                } else {
                                    return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                        5031,
                                        "The transmitted parameters are incorrect",
                                    ))));
                                }
                                let (_, last) = key.split_at(i + 1);
                                let tab = param.get(k);
                                if let Some(LuaValue::Table(t)) = tab {
                                    let data = t.get::<_, LuaTable>(index);
                                    match data {
                                        Ok(v) => v.set(last, val)?,
                                        Err(_) => {
                                            let temp = lua.create_table()?;
                                            temp.set(last, val)?;
                                            t.set(index, temp)?;
                                        }
                                    }
                                } else {
                                    let temp = lua.create_table()?;
                                    let temp2 = lua.create_table()?;
                                    temp2.set(last, val)?;
                                    temp.set(index, temp2)?;
                                    param.insert(k.to_owned(), LuaValue::Table(temp));
                                }
                            } else {
                                let tab = param.get(k);
                                if let Some(LuaValue::Table(t)) = tab {
                                    t.set(t.len()? + 1, val)?;
                                } else {
                                    let temp = lua.create_table()?;
                                    temp.set(1, val)?;
                                    param.insert(k.to_owned(), LuaValue::Table(temp));
                                }
                            }
                            // let tab = param.get(k);
                            // if let Some(LuaValue::Table(t)) = tab {
                            //     t.set(t.len()? + 1, val)?;
                            // } else {
                            //     let temp = lua.create_table()?;
                            //     temp.set(1, val)?;
                            //     param.insert(k.to_owned(), LuaValue::Table(temp));
                            // }
                        }
                    } else {
                        param.insert(key, LuaValue::String(lua.create_string(&val)?));
                    }
                }
                for (key, val) in param {
                    params_table.set(key, val)?;
                }
                log::info!(
                    "params: {}",
                    serde_json::to_string(&params_table).to_lua_err()?
                );
                Ok(params_table)
            } else {
                if !has_content_type(this.0.headers(), &mime::APPLICATION_WWW_FORM_URLENCODED) {
                    return Ok(params_table);
                }
                let bytes = hyper::body::to_bytes(this.0).await.to_lua_err()?;
                let value = serde_urlencoded::from_bytes::<Vec<(String, String)>>(&bytes)
                    .map_err(WebError::parse_params)
                    .to_lua_err()?;
                // for (key, val) in value {
                //     params_table.set(key, val)?;
                // }

                let mut param: HashMap<String, LuaValue> = HashMap::new();
                for (key, val) in value {
                    let offset = key.find('[');
                    if let Some(o) = offset {
                        let k = key.get(0..o);
                        if let Some(k) = k {
                            let offset = key.find('.');
                            // 表明是对象
                            if let Some(i) = offset {
                                let right_offset = key.find(']');
                                let index;
                                if let Some(r) = right_offset {
                                    let a = key.get((o + 1)..r).unwrap_or("1");
                                    let temp = a.parse::<i32>().to_lua_err()?;
                                    index = temp + 1;
                                } else {
                                    return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                        5031,
                                        "The transmitted parameters are incorrect",
                                    ))));
                                }
                                let (_, last) = key.split_at(i + 1);
                                let tab = param.get(k);
                                if let Some(LuaValue::Table(t)) = tab {
                                    let data = t.get::<_, LuaTable>(index);
                                    match data {
                                        Ok(v) => v.set(last, val)?,
                                        Err(_) => {
                                            let temp = lua.create_table()?;
                                            temp.set(last, val)?;
                                            t.set(index, temp)?;
                                        }
                                    }
                                } else {
                                    let temp = lua.create_table()?;
                                    let temp2 = lua.create_table()?;
                                    temp2.set(last, val)?;
                                    temp.set(index, temp2)?;
                                    param.insert(k.to_owned(), LuaValue::Table(temp));
                                }
                            } else {
                                let tab = param.get(k);
                                if let Some(LuaValue::Table(t)) = tab {
                                    t.set(t.len()? + 1, val)?;
                                } else {
                                    let temp = lua.create_table()?;
                                    temp.set(1, val)?;
                                    param.insert(k.to_owned(), LuaValue::Table(temp));
                                }
                            }
                            // let tab = param.get(k);
                            // if let Some(LuaValue::Table(t)) = tab {
                            //     t.set(t.len()? + 1, val)?;
                            // } else {
                            //     let temp = lua.create_table()?;
                            //     temp.set(1, val)?;
                            //     param.insert(k.to_owned(), LuaValue::Table(temp));
                            // }
                        }
                    } else {
                        param.insert(key, LuaValue::String(lua.create_string(&val)?));
                    }
                }
                for (key, val) in param {
                    params_table.set(key, val)?;
                }
                log::info!(
                    "params: {}",
                    serde_json::to_string(&params_table).to_lua_err()?
                );
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

            let mut param: HashMap<String, LuaValue> = HashMap::new();

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
                    let offset = field_name.find('[');
                    if let Some(o) = offset {
                        let k = field_name.get(0..o);
                        if let Some(k) = k {
                            let tab = param.get(k);
                            if let Some(LuaValue::Table(t)) = tab {
                                t.set(t.len()? + 1, data)?;
                            } else {
                                let temp = lua.create_table()?;
                                temp.set(1, data)?;
                                param.insert(k.to_owned(), LuaValue::Table(temp));
                            }
                        }
                    } else {
                        param.insert(field_name, LuaValue::String(lua.create_string(&data)?));
                    }
                    // if field_name.rfind("[]").is_some() {
                    //     field_name.pop();
                    //     field_name.pop();
                    //     let tab = param.get(&field_name);
                    //     if let Some(LuaValue::Table(t)) = tab {
                    //         t.set(t.len()? + 1, data)?;
                    //     } else {
                    //         let temp = lua.create_table()?;
                    //         temp.set(1, data)?;
                    //         param.insert(field_name, LuaValue::Table(temp));
                    //     }
                    // } else {
                    //     param.insert(field_name, LuaValue::String(lua.create_string(&data)?));
                    // }
                }
            }

            for (key, val) in param {
                form_data.set(key, val)?;
            }
            log::info!(
                "form data: {}",
                serde_json::to_string(&form_data).to_lua_err()?
            );
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
        log::info!(
            "Request -- remote address: {}, method: {}, uri: {}",
            self.1,
            method,
            path
        );

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
                    let (code, message) = return_err_info(err);
                    log::error!("{}", message);
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
            if r.len() >= 2 {
                (r[0].parse::<u16>().unwrap_or(500u16), r[1].to_string())
            } else {
                (500, s)
            }
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
    fast_log::init(Config::new().console().file_split(
        "logs/",
        LogSize::MB(50),
        RollingType::KeepNum(5),
        ZipPacker {},
    ))?;
    log::info!("app start...");
    let args = Args::parse();

    // let lua = Lua::new().into_static();
    let lua;
    unsafe {
        lua = Rc::new(Lua::unsafe_new());
    }
    let lua_clone = lua.clone();

    let globals = lua_clone.globals();

    #[cfg(feature = "mysql")]
    globals.set("mysql_pool", lua.create_proxy::<MysqlPool>()?)?;
    #[cfg(feature = "json")]
    globals.set("table_to_json_str", create_table_to_json_string(&lua)?)?;
    #[cfg(feature = "nanoid")]
    globals.set("nanoid", create_nanoid(&lua)?)?;
    #[cfg(feature = "jwt_simple")]
    globals.set("jwt_hs256", lua.create_proxy::<HS256>()?)?;
    #[cfg(feature = "file")]
    globals.set("file", lua.create_proxy::<File>()?)?;
    #[cfg(feature = "http")]
    globals.set("http", lua.create_proxy::<Http>()?)?;
    #[cfg(feature = "crypto")]
    globals.set("crypto", lua.create_proxy::<LuaCrypto>()?)?;
    #[cfg(feature = "alipay")]
    globals.set("alipay", create_alipay(&lua)?)?;

    globals.set("web_error", create_error(&lua)?)?;
    // globals.set("DATEFORMAT", "timestamp")?;

    // let env = lua.create_table()?;
    // env.set("crypto", LuaCrypto)?;

    let file = tokio::fs::read_to_string(args.file)
        .await
        .expect("read file failed");

    let handler: LuaFunction = lua.load(&file).eval()?;

    let is_ipv4: bool = globals.get("ISIPV4")?;
    let addr = if is_ipv4 {
        let localhost: String = globals.get("LOCALHOST")?;
        localhost.parse()?
    } else {
        let port: u16 = globals.get("PORT")?;
        SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), port)
    };
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
