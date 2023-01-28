mod connection;
mod flags;

use connection::SqliteConnection;
use flags::create_open_flags;
use mlua::prelude::*;
use rusqlite::types::Value as SqliteValue;

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

pub fn lua_value_to_sqlite_value(value: LuaValue) -> LuaResult<SqliteValue> {
    match value {
        LuaValue::Nil => Ok(SqliteValue::Null),
        LuaValue::Boolean(v) => {
            if v {
                Ok(SqliteValue::Integer(1))
            } else {
                Ok(SqliteValue::Integer(0))
            }
        }
        LuaValue::Integer(v) => Ok(SqliteValue::Integer(v)),
        LuaValue::Number(v) => Ok(SqliteValue::Real(v)),
        LuaValue::String(v) => {
            let data = v.to_str()?;
            Ok(SqliteValue::Text(data.to_string()))
        }
        _ => Ok(SqliteValue::Null),
    }
}

pub fn sqlite_value_to_lua_value(lua: &Lua, value: SqliteValue) -> LuaResult<LuaValue> {
    match value {
        SqliteValue::Null => Ok(LuaValue::Nil),
        SqliteValue::Integer(v) => Ok(LuaValue::Integer(v)),
        SqliteValue::Real(v) => Ok(LuaValue::Number(v)),
        SqliteValue::Text(v) => Ok(LuaValue::String(lua.create_string(&v)?)),
        SqliteValue::Blob(_) => Ok(LuaValue::Nil), // 暂时不做任何处理，后期优化
    }
}

pub fn table_to_params(table: LuaTable) -> LuaResult<Vec<SqliteValue>> {
    let mut value: Vec<SqliteValue> = Vec::new();
    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (_, val) = pair?;
        value.push(lua_value_to_sqlite_value(val)?);
    }
    // let data = value.try_into().unwrap();
    Ok(value)
}

#[mlua::lua_module]
pub fn sqlite(lua: &Lua) -> LuaResult<LuaTable> {
    lua.create_table_from([
        ("flags", LuaValue::Table(create_open_flags(lua)?)),
        (
            "connect",
            LuaValue::UserData(lua.create_proxy::<SqliteConnection>()?),
        ),
    ])
}
