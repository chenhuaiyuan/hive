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

    let watcher = RecommendedWatcher::new(
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
    if args.dev {
        let hotfix: LuaFunction = lua
            .load(include_str!("./lua/hotfix.lua"))
            .set_name("hive[hotfix]")
            .unwrap()
            .eval()
            .unwrap();
        let (mut watcher, mut rx) = async_watcher()?;

        // let env_file = tokio::fs::read_to_string(args.env.clone())
        //     .await
        //     .expect("read env file failed");
        // let env_var: LuaTable = lua
        //     .load(&env_file)
        //     .set_name("hive[env]")
        //     .unwrap()
        //     .eval()
        //     .unwrap();

        let mut current_dir = env::current_dir().expect("Failed to determine current directory");
        if args.watch_dir != "." {
            current_dir.push(args.watch_dir);
        }
        watcher.watch(&current_dir, RecursiveMode::Recursive)?;

        // for pairs in env_var.pairs::<String, LuaValue>() {
        //     let (key, val) = pairs?;
        //     if key == "unwatch" {
        //         if let LuaValue::Table(v) = val {
        //             for p in v.pairs::<LuaValue, String>() {
        //                 let (_, val) = p?;
        //                 let dir = current_dir.to_str();
        //                 if let Some(dir) = dir {
        //                     let temp_dir = dir.to_owned() + val.as_str();
        //                     watcher.unwatch(Path::new(&temp_dir))?;
        //                 }
        //             }
        //         }
        //     }
        // }

        while let Some(res) = rx.recv().await {
            let event = res.unwrap();
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
                                    hotfix.call::<_, ()>(mode).unwrap();
                                    lua.unset_named_registry_value("http_handler")
                                        .expect("remove registry value fail");
                                    let file = tokio::fs::read_to_string(args.file.clone())
                                        .await
                                        .expect("read index file failed");

                                    let handler: LuaFunction =
                                        lua.load(&file).eval().expect("load lua code fail");
                                    lua.set_named_registry_value("http_handler", handler)
                                        .expect("set registry value fail");
                                }
                            }
                        }
                        // let res = lua.unload(path.to_str().unwrap_or("default"));
                        // if res.is_ok() {
                        //     println!("unload file: {path:?}");
                        //     let fun = lua.load(&path).into_function().unwrap();
                        //     lua.load_from_function::<_, LuaTable>(
                        //         path.to_str().unwrap_or("default"),
                        //         fun,
                        //     )
                        //     .unwrap();
                        // }
                    } else {
                        lua.unset_named_registry_value("http_handler")
                            .expect("remove registry value fail");
                        let file = tokio::fs::read_to_string(args.file.clone())
                            .await
                            .expect("read file failed");

                        let handler: LuaFunction =
                            lua.load(&file).eval().expect("load lua code fail");
                        lua.set_named_registry_value("http_handler", handler)
                            .expect("set registry value fail");
                    }
                }
            }
        }
        // let mut watcher = RecommendedWatcher::new(
        //     move |res: Result<Event, Error>| {
        //         let event = res.unwrap();
        //         if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
        //             lua.unset_named_registry_value("http_handler")
        //                 .expect("remove registry value fail");
        //             let file = fs::read_to_string(args.file.clone()).expect("read file failed");

        //             let handler: LuaFunction = lua.load(&file).eval().expect("load lua code fail");
        //             lua.set_named_registry_value("http_handler", handler)
        //                 .expect("set registry value fail");
        //         }
        //     },
        //     notify::Config::default(),
        // )
        // .expect("Failed to initialize inotify");
        // let current_dir = env::current_dir().expect("Failed to determine current directory");
        // watcher
        //     .watch(&current_dir, RecursiveMode::Recursive)
        //     .expect("Failed to add inotify watch");
        // println!("Watching current directory for activity...");
    }
    Ok(())
}
