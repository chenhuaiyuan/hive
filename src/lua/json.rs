use mlua::prelude::*;

pub fn create_lua_value_to_json_string(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, value: LuaValue| serde_json::to_string(&value).to_lua_err())
}
