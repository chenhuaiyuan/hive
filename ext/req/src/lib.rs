mod req;
mod request;
mod response;

use mlua::prelude::*;
use req::{
    create_delete, create_get, create_head, create_patch, create_post, create_proxy, create_put,
    create_redirect_auth_headers, create_req,
};
use serde_json::Map;
use serde_json::Value as JsonValue;

pub(crate) fn lua_is_array(table: LuaTable) -> LuaResult<bool> {
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

pub(crate) fn lua_value_to_json_value(val: LuaValue) -> LuaResult<JsonValue> {
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
                for pair in v.pairs::<LuaString, LuaValue>() {
                    let (key, val) = pair?;
                    let k = key.to_str()?;
                    map.insert(k.to_string(), lua_value_to_json_value(val)?);
                }
                Ok(JsonValue::from(map))
            }
        }
        _ => Ok(JsonValue::Null),
    }
}

#[mlua::lua_module]
fn req(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("req", create_req(lua)?)?;
    exports.set("proxy", create_proxy(lua)?)?;
    exports.set("redirect_auth_headers", create_redirect_auth_headers(lua)?)?;
    exports.set("delete", create_delete(lua)?)?;
    exports.set("get", create_get(lua)?)?;
    exports.set("head", create_head(lua)?)?;
    exports.set("patch", create_patch(lua)?)?;
    exports.set("post", create_post(lua)?)?;
    exports.set("put", create_put(lua)?)?;
    Ok(exports)
}
