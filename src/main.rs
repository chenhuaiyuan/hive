mod error;
mod file_data;
mod init_object;
#[cfg(feature = "lua")]
mod lua;
mod request;

#[cfg(feature = "lua")]
use crate::error::create_error;

use crate::error::Result as WebResult;
use crate::init_object::create_object;
#[cfg(feature = "lua")]
use crate::lua::notify::async_watch;
#[cfg(feature = "lua")]
use crate::lua::server::create_server;
#[cfg(feature = "lua")]
use crate::lua::service::MakeSvc;
#[cfg(feature = "ws")]
use crate::lua::ws::create_message;
#[cfg(feature = "lua")]
use crate::lua::{file_data::FileData, json::create_table_to_json_string};
use clap::Parser;
use fast_log::{
    config::Config,
    consts::LogSize,
    plugin::{file_split::RollingType, packer::ZipPacker},
};
use futures_util::Future;
use hyper::Server;
#[cfg(feature = "lua")]
use mlua::prelude::*;
use once_cell::sync::Lazy;
use std::fs;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;

pub static HALF_NUM_CPUS: Lazy<usize> = Lazy::new(|| 1.max(num_cpus::get() / 2));

#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// 读取的文件名
    #[cfg(feature = "lua")]
    #[arg(short, long, default_value = "index.lua")]
    file: String,
    #[cfg(feature = "js")]
    #[arg(short, long, default_value = "index.js")]
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

#[cfg(feature = "lua")]
fn lua_run(args: Args) {
    let lua;
    unsafe {
        lua = Arc::new(Lua::unsafe_new());
    }

    let lua_clone = lua.clone();
    let globals = lua_clone.globals();

    let hive = lua.create_table()?;

    #[cfg(feature = "lua")]
    hive.set("table_to_json", create_table_to_json_string(&lua)?)?;
    #[cfg(feature = "lua")]
    hive.set("file_data", lua.create_proxy::<FileData>()?)?;

    hive.set("web_error", create_error(&lua)?)?;
    hive.set("env", lua.create_table_from([("dev", args.dev)])?)?;
    hive.set("version", lua.create_string(env!("CARGO_PKG_VERSION"))?)?;
    hive.set("server", create_server(&lua)?)?;
    #[cfg(feature = "ws")]
    hive.set("ws_message", create_message(&lua)?)?;
    globals.set("hive", hive)?;

    let file = fs::read(args.file.clone()).expect("read file failed");

    let handler: LuaTable = lua.load(&file).eval()?;

    let is_ipv4: bool = handler.get("is_ipv4").unwrap_or(true);
    let localhost: String = handler.get("addr").unwrap_or("127.0.0.1".to_owned());
    let port: u16 = handler.get("port").unwrap_or(3000);
    let addr = if is_ipv4 {
        SocketAddr::new(IpAddr::V4(localhost.parse()?), port)
    } else {
        SocketAddr::new(IpAddr::V6(localhost.parse()?), port)
    };
    println!("Listening on http://{addr}");
    lua.set_named_registry_value("http_handler", handler.get::<_, LuaFunction>("serve")?)?;
    lua.set_named_registry_value("exception", handler.get::<_, LuaFunction>("exception")?)?;
    if args.dev {
        block_on(async {
            let server = Server::bind(&addr)
                .executor(LocalExec)
                .serve(MakeSvc(lua.clone()));
            let local = tokio::task::LocalSet::new();
            let j = tokio::join! {
                async_watch(lua.clone(), args.clone()),
                local.run_until(server)
            };
            j.0.unwrap();
        });
    } else {
        block_on(async {
            let server = Server::bind(&addr)
                .executor(LocalExec)
                .serve(MakeSvc(lua.clone()));
            let local = tokio::task::LocalSet::new();
            local.run_until(server).await.unwrap();
        });
    }
}

#[cfg(feature = "js")]
fn v8_run(args: Args) {
    use v8::{Context, ContextScope, HandleScope, Isolate, Script, String};

    let isolate = &mut Isolate::new(Default::default());

    let scope = &mut HandleScope::new(isolate);
    let context = Context::new(scope);
    let scope = &mut ContextScope::new(scope, context);

    let file = fs::read_to_string(args.file.clone()).expect("read file failed");
    let code = String::new(scope, &file).unwrap();
    let script = Script::compile(scope, code, None).unwrap();
    script.run(scope).unwrap();
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

    #[cfg(feature = "lua")]
    lua_run(args);
    #[cfg(feature = "js")]
    v8_run(args);
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
