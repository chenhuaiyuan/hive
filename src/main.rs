mod error;
mod file_data;
#[cfg(feature = "create_object")]
mod init_project;
#[cfg(feature = "js")]
mod js;
#[cfg(feature = "lua")]
mod lua;
mod request;

use crate::error::Result as WebResult;
#[cfg(feature = "create_object")]
use crate::init_project::create_project;

#[cfg(feature = "lua_hotfix")]
use crate::lua::notify::async_watch;

#[cfg(feature = "lua")]
use crate::lua::service::MakeSvc;

use clap::Parser;
#[cfg(feature = "hive_log")]
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
    /// 自定义参数,多个参数之间用“&”分割，例如：aaa=123&b=456
    #[arg(long)]
    custom_params: Option<String>,
}

#[cfg(feature = "lua")]
async fn lua_run(args: Args) -> WebResult<()> {
    use crate::lua::hive_func::add_hive_func;
    #[cfg(not(feature = "lua_hotfix"))]
    use crate::lua::router::HiveRouter;

    let lua: Arc<Lua> = unsafe { Arc::new(Lua::unsafe_new()) };

    let lua_clone = lua.clone();
    let globals: LuaTable = lua_clone.globals();
    let hive = add_hive_func(&lua, args.dev, args.custom_params.clone())?;
    globals.set("hive", hive)?;

    let file: Vec<u8> = fs::read(args.file.clone()).expect("read file failed");

    let handler: LuaTable = lua.load(&file).eval_async().await?;

    #[cfg(not(feature = "lua_hotfix"))]
    let router: LuaAnyUserData = handler.get("router")?;
    #[cfg(not(feature = "lua_hotfix"))]
    let router = Some(Arc::new(router.take::<HiveRouter>()?));
    #[cfg(feature = "lua_hotfix")]
    let router = None;
    let is_ipv4: bool = handler.get("is_ipv4").unwrap_or(true);
    let localhost: String = handler.get("addr").unwrap_or("127.0.0.1".to_owned());
    let port: u16 = handler.get("port").unwrap_or(3000);
    let addr: SocketAddr = if is_ipv4 {
        SocketAddr::new(IpAddr::V4(localhost.parse()?), port)
    } else {
        SocketAddr::new(IpAddr::V6(localhost.parse()?), port)
    };
    println!("Listening on http://{addr}");
    let http_handler = lua.create_registry_value(handler.get::<_, LuaFunction>("serve")?)?;
    let http_handler = Arc::new(http_handler);
    let exception = lua.create_registry_value(handler.get::<_, LuaFunction>("exception")?)?;
    let exception = Arc::new(exception);
    if args.dev {
        let make_svc = MakeSvc {
            lua: lua.clone(),
            handler: Some(http_handler),
            exception,
            router,
        };
        let server = Server::bind(&addr).executor(LocalExec).serve(make_svc);
        let local = tokio::task::LocalSet::new();
        #[cfg(feature = "lua_hotfix")]
        {
            let j = tokio::join! {
                async_watch(lua.clone(), args.clone()),
                local.run_until(server)
            };
            j.0.unwrap();
        }
        #[cfg(not(feature = "lua_hotfix"))]
        {
            local.run_until(server).await.unwrap();
        }
    } else {
        let make_svc = MakeSvc {
            lua: lua.clone(),
            handler: Some(http_handler),
            exception,
            router,
        };
        let server = Server::bind(&addr).executor(LocalExec).serve(make_svc);
        let local = tokio::task::LocalSet::new();
        local.run_until(server).await.unwrap();
    }
    Ok(())
}

#[cfg(feature = "js")]
async fn v8_run(args: Args) -> WebResult<()> {
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

#[cfg(feature = "hive_log")]
fn hive_logs(args: &Args) -> WebResult<()> {
    if args.dev {
        fast_log::init(Config::new().console().file_split(
            "logs/",
            LogSize::MB(50),
            RollingType::KeepNum(5),
            ZipPacker {},
        ))?;
        log::info!("env: dev mode");
    } else if let Some(_object_name) = &args.create {
        #[cfg(feature = "create_object")]
        {
            create_project(_object_name)?;
        }
        #[cfg(not(feature = "create_object"))]
        {
            println!("没有开启此功能");
        }
        return Ok(());
    } else {
        fast_log::init(Config::new().file_split(
            "logs/",
            LogSize::MB(50),
            RollingType::KeepNum(5),
            ZipPacker {},
        ))?;
    }
    Ok(())
}

// #[tokio::main(flavor = "multi_thread")]
fn main() -> WebResult<()> {
    let args = Args::parse();

    #[cfg(feature = "hive_log")]
    hive_logs(&args)?;

    log::info!("app start...");

    #[cfg(feature = "lua")]
    block_on(lua_run(args))?;
    #[cfg(feature = "js")]
    block_on(v8_run(args))?;
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

fn block_on<F: Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(*HALF_NUM_CPUS)
        .build()
        .unwrap()
        .block_on(f)
}
