#[cfg(feature = "alipay")]
pub mod alipay;
#[cfg(feature = "crypto")]
pub mod crypto;
#[cfg(feature = "file")]
pub mod file;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "jwt_simple")]
pub mod jwt_simple;
#[cfg(feature = "http")]
pub mod lua_http;
#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "nanoid")]
pub mod nanoid;

use serde_json::Map;

use mlua::prelude::*;
use serde_json::Value as JsonValue;

pub(crate) fn json_value_to_lua_value(val: JsonValue, lua: &Lua) -> LuaResult<LuaValue> {
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
                table.set(i, json_value_to_lua_value(val, lua)?)?;
                i += 1;
            }
            return Ok(LuaValue::Table(table));
        }
        JsonValue::Object(v) => {
            let table = lua.create_table()?;
            for (key, val) in v {
                table.set(key, json_value_to_lua_value(val, lua)?)?;
            }
            return Ok(LuaValue::Table(table));
        }
    }
}

pub(crate) fn lua_value_to_json_value(val: LuaValue, _lua: &Lua) -> LuaResult<JsonValue> {
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
