use super::lua_value_to_json_value;
use mlua::prelude::*;

pub fn create_table_to_json_string(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, table: LuaTable| {
        let data = lua_value_to_json_value(LuaValue::Table(table), lua)?;
        let json = serde_json::to_string(&data).to_lua_err()?;
        Ok(json)
    })
}
