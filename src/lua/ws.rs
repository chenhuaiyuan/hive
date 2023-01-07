use crate::error::Error as WebError;
use mlua::prelude::*;
use std::{path::Path, sync::Arc};
use tokio::fs;
use tungstenite::Message;

pub struct WSMessage(pub Message);

impl LuaUserData for WSMessage {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("text", |_, text: String| {
            let msg = Message::text(text);
            Ok(WSMessage(msg))
        });
        _methods.add_async_function("binary_from_file_path", |_, path: String| async move {
            let bin = fs::read(Path::new(&path)).await;
            if let Ok(binary) = bin {
                let msg = Message::binary(binary);
                Ok(WSMessage(msg))
            } else {
                Err(LuaError::ExternalError(Arc::new(WebError::new(
                    5051,
                    "Fail To Read File",
                ))))
            }
        });
        _methods.add_method("is_text", |_, this, ()| Ok(this.0.is_text()));
        _methods.add_method("is_binary", |_, this, ()| Ok(this.0.is_binary()));
        _methods.add_method("is_ping", |_, this, ()| Ok(this.0.is_ping()));
        _methods.add_method("is_pong", |_, this, ()| Ok(this.0.is_pong()));
        _methods.add_method("is_close", |_, this, ()| Ok(this.0.is_close()));
        _methods.add_method("len", |_, this, ()| Ok(this.0.len()));
        _methods.add_method("is_empty", |_, this, ()| Ok(this.0.is_empty()));
        _methods.add_function("into_data", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            Ok(this.0.into_data())
        });
        _methods.add_function("into_text", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            this.0.into_text().to_lua_err()
        });
        _methods.add_method("to_text", |_, this, ()| {
            let text = this.0.to_text().to_lua_err()?;
            Ok(text.to_owned())
        });
        _methods.add_async_function(
            "save_binary",
            |_, (this, file_path): (LuaAnyUserData, String)| async move {
                let this = this.take::<Self>()?;
                if this.0.is_binary() {
                    fs::write(Path::new(&file_path), this.0.into_data())
                        .await
                        .to_lua_err()?;
                    Ok(())
                } else {
                    Err(LuaError::ExternalError(Arc::new(WebError::new(
                        5052,
                        "Binary File Save Failed",
                    ))))
                }
            },
        );
    }
}

pub fn create_message(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    lua.create_proxy::<WSMessage>()
}
