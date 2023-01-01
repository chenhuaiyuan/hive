use mlua::prelude::*;
use serde_json::Map;
use serde_json::Value as JsonValue;

pub fn lua_is_array(table: LuaTable) -> LuaResult<bool> {
    let mut is_array = true;
    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, _) = pair?;
        if key.type_name() != "integer" {
            is_array = false;
            break;
        }
    }
    Ok(is_array)
}

pub fn json_value_to_lua_value(lua: &Lua, val: JsonValue) -> LuaResult<LuaValue> {
    match val {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(v) => Ok(LuaValue::Boolean(v)),
        JsonValue::Number(v) => {
            if v.is_i64() {
                let num = v.as_i64();
                if let Some(num) = num {
                    return Ok(LuaValue::Integer(num));
                }
            } else if v.is_u64() {
                let num = v.as_u64();
                if let Some(num) = num {
                    return Ok(LuaValue::Number(num as f64));
                }
            } else if v.is_f64() {
                let num = v.as_f64();
                if let Some(num) = num {
                    return Ok(LuaValue::Number(num));
                }
            }
            return Ok(LuaValue::Integer(0));
        }
        JsonValue::String(v) => {
            let s = lua.create_string(&v)?;
            return Ok(LuaValue::String(s));
        }
        JsonValue::Array(v) => {
            let table = lua.create_table()?;
            let mut i = 1;
            for val in v {
                table.set(i, json_value_to_lua_value(lua, val)?)?;
                i += 1;
            }
            return Ok(LuaValue::Table(table));
        }
        JsonValue::Object(v) => {
            let table = lua.create_table()?;
            for (key, val) in v {
                table.set(key, json_value_to_lua_value(lua, val)?)?;
            }
            return Ok(LuaValue::Table(table));
        }
    }
}

pub fn lua_value_to_json_value(val: LuaValue) -> LuaResult<JsonValue> {
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
            let is_array = lua_is_array(v.clone())?;
            if is_array {
                let mut arr: Vec<JsonValue> = Vec::new();
                for pair in v.pairs::<LuaValue, LuaValue>() {
                    let (_, val) = pair?;
                    arr.push(lua_value_to_json_value(val)?);
                }
                Ok(JsonValue::from(arr))
            } else {
                let mut map: Map<String, JsonValue> = Map::new();
                for pair in v.pairs::<String, LuaValue>() {
                    let (key, val) = pair?;
                    map.insert(key, lua_value_to_json_value(val)?);
                }
                Ok(JsonValue::from(map))
            }
        }
        _ => Ok(JsonValue::Null),
    }
}