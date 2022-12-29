use mlua::prelude::*;

pub fn create_router(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.load(include_str!("./lua/router.lua")).into_function()
}
