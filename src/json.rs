use mlua::prelude::*;
use serde_json::{Map, Value as JsonValue};

pub fn create_table_to_json_string(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, table: LuaTable| {
        let data = lua_value_to_json_value(LuaValue::Table(table), lua)?;
        let json = serde_json::to_string(&data).to_lua_err()?;
        Ok(json)
    })
}

fn lua_value_to_json_value(val: LuaValue, _lua: &Lua) -> LuaResult<JsonValue> {
    match val {
        LuaValue::Nil => Ok(JsonValue::Null),
        LuaValue::Boolean(v) => Ok(JsonValue::Bool(v)),
        LuaValue::Integer(v) => Ok(JsonValue::from(v)),
        LuaValue::Number(v) => Ok(JsonValue::from(v)),
        LuaValue::String(v) => {
            let data = v.to_str()?;
            Ok(JsonValue::from(data))
        }
        LuaValue::Table(v) => {
            let mut is_array = true;
            for pair in v.clone().pairs::<LuaValue, LuaValue>() {
                let (key, _) = pair?;
                if key.type_name() != "integer" {
                    is_array = false;
                    break;
                }
            }
            if is_array {
                let mut arr: Vec<JsonValue> = Vec::new();
                for pair in v.pairs::<LuaValue, LuaValue>() {
                    let (_, val) = pair?;
                    arr.push(lua_value_to_json_value(val, _lua)?);
                }
                Ok(JsonValue::from(arr))
            } else {
                let mut map: Map<String, JsonValue> = Map::new();
                for pair in v.pairs::<LuaString, LuaValue>() {
                    let (key, val) = pair?;
                    let k = key.to_str()?;
                    map.insert(k.to_string(), lua_value_to_json_value(val, _lua)?);
                }
                Ok(JsonValue::from(map))
            }
        }
        _ => Ok(JsonValue::Null),
    }
}
