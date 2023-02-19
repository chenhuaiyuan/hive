use crate::error::Result;
use mlua::prelude::*;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::env;
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver};

use crate::Args;

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = channel(1);

    let watcher: notify::FsEventWatcher = RecommendedWatcher::new(
        move |res| {
            Runtime::new().unwrap().block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        notify::Config::default(),
    )?;
    Ok((watcher, rx))
}

pub async fn async_watch(lua: Arc<Lua>, args: Args) -> Result<()> {
    let hotfix: LuaFunction = lua
        .load(include_str!("./hotfix.lua"))
        .set_name("hive[hotfix]")?
        .eval()?;
    let (mut watcher, mut rx) = async_watcher()?;

    let mut current_dir: std::path::PathBuf =
        env::current_dir().expect("Failed to determine current directory");
    if args.watch_dir != "." {
        current_dir.push(args.watch_dir.clone());
    }
    watcher.watch(&current_dir, RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        let event = res?;
        if event.kind.is_modify() {
            for path in event.paths {
                if path != Path::new(&args.file) {
                    let current_dir_len = current_dir.as_os_str().len() + 1;
                    let p = path.to_str();
                    if let Some(p) = p {
                        let file = p.get(current_dir_len..);
                        if let Some(file) = file {
                            let file = file.replace('/', ".");
                            let data = file.rsplit_once('.');
                            if let Some((mode, _)) = data {
                                let mut m;
                                if args.watch_dir != "." {
                                    let watch_dir = args.watch_dir.clone().replace('/', ".");
                                    m = watch_dir + ".";
                                    m += mode;
                                } else {
                                    m = mode.to_string();
                                }
                                hotfix.call::<_, ()>(m)?;
                                let file = tokio::fs::read(args.file.clone()).await?;

                                let handler: LuaTable = lua.load(&file).eval()?;
                                lua.set_named_registry_value(
                                    "http_handler",
                                    handler.get::<_, LuaFunction>("serve")?,
                                )?;
                                lua.set_named_registry_value(
                                    "exception",
                                    handler.get::<_, LuaFunction>("exception")?,
                                )?;
                            }
                        }
                    }
                } else {
                    let file: Vec<u8> = tokio::fs::read(args.file.clone()).await?;

                    let handler: LuaTable = lua.load(&file).eval()?;
                    lua.set_named_registry_value(
                        "http_handler",
                        handler.get::<_, LuaFunction>("serve")?,
                    )?;
                    lua.set_named_registry_value(
                        "exception",
                        handler.get::<_, LuaFunction>("exception")?,
                    )?;
                }
            }
        }
    }
    Ok(())
}
