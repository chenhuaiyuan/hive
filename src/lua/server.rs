use mlua::prelude::*;

pub fn create_server(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.load(include_str!("server.lua")).into_function()
}
