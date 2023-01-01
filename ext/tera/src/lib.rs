mod context;
mod tera;
use crate::context::create_context;
use crate::tera::{create_escape_html, create_tera};
use mlua::prelude::*;

#[mlua::lua_module]
fn tera(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("context", create_context(lua)?)?;
    exports.set("tera", create_tera(lua)?)?;
    exports.set("escape_html", create_escape_html(lua)?)?;
    Ok(exports)
}
