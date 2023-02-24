mod error;
mod file_data;
mod init_object;
#[cfg(feature = "js")]
mod js;
#[cfg(feature = "lua")]
mod lua;
mod request;

#[cfg(feature = "lua")]
use crate::error::create_error;

use crate::error::Result as WebResult;
use crate::init_object::create_object;
#[cfg(feature = "lua")]
use crate::lua::mysql_async::create_mysql;
// use crate::lua::mysql_sqlx::create_sqlx;
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
use hyper::Server;
#[cfg(feature = "lua")]
use mlua::prelude::*;
use std::fs;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::Arc;

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
    /// 自定义参数,多个参数之间用“&”分割，例如：aaa=123&b=456
    #[arg(long)]
    custom_params: Option<String>,
}

// 自定义参数处理
// 例如：a=123&b=456
fn custom_params_parse(lua: &Lua, params: String) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    let params: Vec<&str> = params.split('&').collect();
    for param in params {
        let p: Vec<&str> = param.split('=').collect();
        table.set(p[0], p[1])?;
    }
    Ok(table)
}

#[cfg(feature = "lua")]
async fn lua_run(args: Args) -> WebResult<()> {
    use crate::lua::router::create_router;

    let lua: Arc<Lua> = unsafe { Arc::new(Lua::unsafe_new()) };

    let lua_clone = lua.clone();
    let globals: LuaTable = lua_clone.globals();

    let hive: LuaTable = lua.create_table()?;

    hive.set("table_to_json", create_table_to_json_string(&lua)?)?;
    hive.set("file_data", lua.create_proxy::<FileData>()?)?;
    hive.set("web_error", create_error(&lua)?)?;
    if let Some(ref custom_params) = args.custom_params {
        let env = custom_params_parse(&lua, custom_params.clone())?;
        env.set("dev", args.dev)?;
        hive.set("env", env)?;
    } else {
        hive.set("env", lua.create_table_from([("dev", args.dev)])?)?;
    }
    hive.set("version", lua.create_string(env!("CARGO_PKG_VERSION"))?)?;
    hive.set("server", create_server(&lua)?)?;
    #[cfg(feature = "ws")]
    hive.set("ws_message", create_message(&lua)?)?;
    hive.set("mysql", create_mysql(&lua)?)?;
    hive.set("router", create_router(&lua)?)?;
    globals.set("hive", hive)?;

    let file: Vec<u8> = fs::read(args.file.clone()).expect("read file failed");

    let handler: LuaTable = lua.load(&file).eval_async().await?;

    let is_ipv4: bool = handler.get("is_ipv4").unwrap_or(true);
    let localhost: String = handler.get("addr").unwrap_or("127.0.0.1".to_owned());
    let port: u16 = handler.get("port").unwrap_or(3000);
    let addr: SocketAddr = if is_ipv4 {
        SocketAddr::new(IpAddr::V4(localhost.parse()?), port)
    } else {
        SocketAddr::new(IpAddr::V6(localhost.parse()?), port)
    };
    println!("Listening on http://{addr}");
    lua.set_named_registry_value("http_handler", handler.get::<_, LuaFunction>("serve")?)?;
    lua.set_named_registry_value("exception", handler.get::<_, LuaFunction>("exception")?)?;
    if args.dev {
        let server = Server::bind(&addr)
            .executor(LocalExec)
            .serve(MakeSvc(lua.clone()));
        let local = tokio::task::LocalSet::new();
        let j = tokio::join! {
            async_watch(lua.clone(), args.clone()),
            local.run_until(server)
        };
        j.0.unwrap();
    } else {
        let server = Server::bind(&addr)
            .executor(LocalExec)
            .serve(MakeSvc(lua.clone()));
        let local = tokio::task::LocalSet::new();
        local.run_until(server).await.unwrap();
    }
    Ok(())
}

#[cfg(feature = "js")]
fn v8_run(args: Args) -> WebResult<()> {
    // use v8::{Context, ContextScope, HandleScope, Isolate, Script, String, TryCatch, V8};

    // use crate::js::server::create_server;

    // let platform = v8::new_default_platform(0, false).make_shared();
    // v8::V8::initialize_platform(platform);
    // v8::V8::initialize();

    // {
    //     let isolate = &mut v8::Isolate::new(Default::default());
    //     let scope = &mut v8::HandleScope::new(isolate);

    //     let code = fs::read_to_string(args.file.clone()).expect("read file failed");
    //     let code = v8::String::new(scope, &code).unwrap();
    //     let source = v8::Global::new(scope, code);

    //     let script = v8::Script::compile(scope, code, None).unwrap();
    //     let result = script.run(scope).unwrap();
    //     let result = result.to_string(scope).unwrap();
    //     println!("result: {}", result.to_rust_string_lossy(scope));
    // }

    // unsafe {
    //     v8::V8::dispose();
    // }
    // v8::V8::dispose_platform();
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> WebResult<()> {
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
        fast_log::init(Config::new().file_split(
            "logs/",
            LogSize::MB(50),
            RollingType::KeepNum(5),
            ZipPacker {},
        ))?;
    }
    log::info!("app start...");

    #[cfg(feature = "lua")]
    lua_run(args).await?;
    #[cfg(feature = "js")]
    v8_run(args)?;
    Ok(())
}

#[cfg(feature = "lua")]
#[derive(Clone, Copy, Debug)]
pub struct LocalExec;
#[cfg(feature = "lua")]
impl<F> hyper::rt::Executor<F> for LocalExec
where
    F: std::future::Future + 'static, // not requiring `Send`
{
    fn execute(&self, fut: F) {
        tokio::task::spawn_local(fut);
    }
}
