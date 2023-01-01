mod error;
mod init_object;
mod notify;
mod router;
#[cfg(any(feature = "file_data", feature = "json"))]
mod utils;

use crate::error::{create_error, Error as WebError, Result as WebResult};
use crate::init_object::create_object;
use crate::notify::async_watch;
use crate::router::create_router;
use crate::utils::{
    file_data::FileData, json::create_table_to_json_string, lua_request::LuaRequest,
};
use clap::Parser;
use fast_log::{
    config::Config,
    consts::LogSize,
    plugin::{file_split::RollingType, packer::ZipPacker},
};
use futures_util::Future;
// use http::{header::HeaderValue, header::CONTENT_TYPE};
use hyper::service::Service;
use hyper::{server::conn::AddrStream, Body, Request, Response, Server};
use mlua::prelude::*;
use once_cell::sync::Lazy;
use std::fs;
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv6Addr};
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;

pub static HALF_NUM_CPUS: Lazy<usize> = Lazy::new(|| 1.max(num_cpus::get() / 2));

struct Svc(Arc<Lua>, SocketAddr);

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
        let lua_req = LuaRequest::new(req, self.1);
        log::info!(
            "Request -- remote address: {}, method: {}, uri: {}",
            self.1,
            method,
            path
        );

        Box::pin(async move {
            let handler: LuaFunction = lua.named_registry_value("http_handler")?;

            match handler
                .call_async::<_, LuaValue>((method, path, lua_req))
                .await
            {
                Ok(lua_resp) => match lua_resp {
                    LuaValue::Integer(v) => Ok(Response::new(Body::from(v.to_string()))),
                    LuaValue::Number(v) => Ok(Response::new(Body::from(v.to_string()))),
                    LuaValue::String(v) => Ok(Response::new(Body::from(v.to_str()?.to_string()))),
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
                    let exception: LuaFunction = lua.named_registry_value("exception")?;
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

                            let body = v
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

struct MakeSvc(Arc<Lua>);

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

#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// 读取的文件名
    #[arg(short, long, default_value = "index.lua")]
    file: String,
    /// 是否开启dev模式，默认值：false
    #[arg(short, long, default_value_t = false)]
    dev: bool,
    /// 设置监视路径，默认当前路径
    #[arg(short, long, default_value = ".")]
    watch_dir: String,
    /// 创建项目，举例：hive --create test
    #[arg(long)]
    create: Option<String>,
    /// release环境下热重载，此功能还未实现
    #[arg(long, default_value_t = false)]
    reload: bool,
}

fn main() -> WebResult<()> {
    let args = Args::parse();
    if args.dev {
        fast_log::init(Config::new().console().file_split(
            "logs/",
            LogSize::MB(50),
            RollingType::KeepNum(5),
            ZipPacker {},
        ))?;
        log::info!("env: dev mode");
    } else if let Some(object_name) = args.create {
        create_object(object_name)?;
        return Ok(());
    } else {
        fast_log::init(Config::new().console().file_split(
            "logs/",
            LogSize::MB(50),
            RollingType::KeepNum(5),
            ZipPacker {},
        ))?;
    }
    log::info!("app start...");

    // let lua = Lua::new().into_static();
    let lua;
    unsafe {
        lua = Arc::new(Lua::unsafe_new());
    }

    let lua_clone = lua.clone();
    let globals = lua_clone.globals();

    let hive = lua.create_table()?;

    #[cfg(feature = "json")]
    hive.set("table_to_json", create_table_to_json_string(&lua)?)?;
    #[cfg(feature = "file_data")]
    hive.set("file_data", lua.create_proxy::<FileData>()?)?;

    hive.set("web_error", create_error(&lua)?)?;
    hive.set("router", create_router(&lua)?)?;
    hive.set("env", lua.create_table_from([("dev", args.dev)])?)?;
    hive.set("version", lua.create_string(env!("CARGO_PKG_VERSION"))?)?;
    globals.set("hive", hive)?;
    // globals.set("DATEFORMAT", "timestamp")?;

    // let env = lua.create_table()?;
    // env.set("crypto", LuaCrypto)?;

    let file = fs::read(args.file.clone()).expect("read file failed");

    let (handler, exception): (LuaFunction, LuaFunction) = lua.load(&file).eval()?;

    let is_ipv4: bool = globals.get("ISIPV4").unwrap_or(true);
    let addr = if is_ipv4 {
        let localhost: String = globals
            .get("LOCALHOST")
            .unwrap_or("127.0.0.1:3000".to_owned());
        localhost.parse()?
    } else {
        let port: u16 = globals.get("PORT").unwrap_or(3000);
        SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), port)
    };
    println!("Listening on http://{addr}");
    lua.set_named_registry_value("http_handler", handler)?;
    lua.set_named_registry_value("exception", exception)?;
    block_on(async {
        let server = Server::bind(&addr)
            .executor(LocalExec)
            .serve(MakeSvc(lua.clone()));
        let local = tokio::task::LocalSet::new();
        let j = tokio::join! {
            async_watch(lua, args.clone()),
            local.run_until(server)
        };
        j.0.unwrap();
    });
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

fn block_on<F: Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(*HALF_NUM_CPUS)
        .build()
        .unwrap()
        .block_on(f)
}
