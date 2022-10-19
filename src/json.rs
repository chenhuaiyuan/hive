use mlua::prelude::*;
// use serde_json::value as JsonValue;

pub fn create_table_to_json_string(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, table: LuaTable| {
        let json = serde_json::to_string(&table).to_lua_err()?;
        Ok(json)
    })
}

// fn lua_value_to_json_value(val: LuaValue, lua: &Lua) -> LuaResult<JsonValue> {
//     match val {
//         LuaValue::Nil => Ok(JsonValue::Null),
//         LuaValue::Boolean(v) => Ok(JsonValue::Bool(v)),
//         LuaValue::Integer(v) => Ok(JsonValue::from(v)),
//         LuaValue::Number(v) => Ok(JsonValue::from(v)),
//         LuaValue::String(v) => {
//             let data = v.to_str()?;
//             Ok(JsonValue::from_str(data).to_lua_err()?)
//         }
//         _ => Ok(JsonValue::Null),
//     }
// }
