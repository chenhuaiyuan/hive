use hive_base::lua_value_to_json_value;
use mlua::prelude::*;
use serde_json::Value as JsonValue;

pub fn create_table_to_json_string(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, table: LuaTable| {
        let data: JsonValue = lua_value_to_json_value(LuaValue::Table(table))?;
        let json: String = serde_json::to_string(&data).to_lua_err()?;
        Ok(json)
    })
}
