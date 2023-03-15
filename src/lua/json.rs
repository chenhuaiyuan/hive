use mlua::prelude::*;

pub fn create_lua_value_to_json_string(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, value: LuaValue| serde_json::to_string(&value).to_lua_err())
}

pub fn create_empty_array(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, ()| lua.to_value(&serde_json::json!([])))
}
