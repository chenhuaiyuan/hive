mod req;
mod request;
mod response;

use mlua::prelude::*;
use req::{
    create_delete, create_get, create_head, create_patch, create_post, create_proxy, create_put,
    create_redirect_auth_headers, create_req,
};

#[mlua::lua_module]
fn req(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("req", create_req(lua)?)?;
    exports.set("proxy", create_proxy(lua)?)?;
    exports.set("redirect_auth_headers", create_redirect_auth_headers(lua)?)?;
    exports.set("delete", create_delete(lua)?)?;
    exports.set("get", create_get(lua)?)?;
    exports.set("head", create_head(lua)?)?;
    exports.set("patch", create_patch(lua)?)?;
    exports.set("post", create_post(lua)?)?;
    exports.set("put", create_put(lua)?)?;
    Ok(exports)
}
