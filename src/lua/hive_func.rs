use crate::error::{create_error, Result};
#[cfg(feature = "lua_file_data")]
use crate::lua::file_data::FileData;
#[cfg(feature = "mysql")]
use crate::lua::mysql_async::create_mysql;
#[cfg(feature = "ws")]
use crate::lua::ws::create_message;
use crate::lua::{
    json::create_empty_array, response::HiveResponseBuilder, router::create_router,
    server::create_server,
};
use mlua::prelude::*;

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

pub fn add_hive_func(
    lua: &Lua,
    args_dev: bool,
    args_custom_params: Option<String>,
) -> Result<LuaTable> {
    let hive: LuaTable = lua.create_table()?;
    hive.set("empty_array", create_empty_array(lua)?)?;
    #[cfg(feature = "lua_file_data")]
    hive.set("file_data", lua.create_proxy::<FileData>()?)?;
    hive.set("web_error", create_error(lua)?)?;
    if let Some(ref custom_params) = args_custom_params {
        let env = custom_params_parse(lua, custom_params.to_string())?;
        env.set("dev", args_dev)?;
        hive.set("env", env)?;
    } else {
        hive.set("env", lua.create_table_from([("dev", args_dev)])?)?;
    }
    hive.set("version", lua.create_string(env!("CARGO_PKG_VERSION"))?)?;
    hive.set("server", create_server(lua)?)?;
    #[cfg(feature = "ws")]
    hive.set("ws_message", create_message(lua)?)?;
    #[cfg(feature = "mysql")]
    hive.set("mysql", create_mysql(lua)?)?;
    hive.set("router", create_router(lua)?)?;
    hive.set("response", lua.create_proxy::<HiveResponseBuilder>()?)?;
    Ok(hive)
}
