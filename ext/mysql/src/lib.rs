mod error;
use dateparser::DateTimeUtc;
use error::Error as WebError;
use mlua::prelude::*;
use mysql::{prelude::Queryable, Opts, Pool, Row, Value as MysqlValue};
use std::sync::Arc;

macro_rules! row_to_table {
    ($row:expr, $lua:ident) => {{
        let columns = $row.columns();
        let table = $lua.create_table()?;
        for column in columns.iter() {
            let key = column.name_str();
            let val: Option<MysqlValue> = $row.take(key.to_string().as_str());
            let v: LuaValue;
            if let Some(val) = val {
                v = mysql_value_to_lua_value(val, $lua)?;
            } else {
                v = LuaValue::Nil;
            }
            table.set(String::from(key), v)?;
        }
        table
    }};
}

#[derive(Clone)]
pub struct MysqlPool(Pool);

impl LuaUserData for MysqlPool {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function(
            "new",
            |_, (username, password, address, database): (String, String, String, Option<String>)| {
                let mysql_url;
                if let Some(db) = database {
                    mysql_url = format!("mysql://{}:{}@{}/{}", username, password, address, db);
                }else {
                    mysql_url = format!("mysql://{}:{}@{}", username, password, address);
                }
                let opts = Opts::from_url(&mysql_url).to_lua_err()?;
                let pool = Pool::new(opts).to_lua_err()?;
                Ok(MysqlPool(pool))
            },
        );
        _methods.add_method("query", |lua, this, sql: String| {
            let query_data = lua.create_table()?;

            let mut conn = this.0.get_conn().to_lua_err()?;

            let rows: Vec<Row> = conn.query(sql).to_lua_err()?;
            let mut i = 1;
            for mut row in rows {
                let data = row_to_table!(row, lua);
                query_data.set(i, data)?;
                i += 1;
            }

            Ok(query_data)
        });
        _methods.add_method("query_first", |lua, this, sql: String| {
            let mut conn = this.0.get_conn().to_lua_err()?;

            let row: Option<Row> = conn.query_first(sql).to_lua_err()?;
            if let Some(mut row) = row {
                let data = row_to_table!(row, lua);
                Ok(data)
            } else {
                lua.create_table()
            }
        });
        _methods.add_method(
            "exec",
            |lua, this, (sql, params): (String, LuaMultiValue)| {
                let mut conn = this.0.get_conn().to_lua_err()?;
                if params.is_empty() {
                    let query_data = lua.create_table()?;
                    let rows: Vec<Row> = conn.exec(sql, ()).to_lua_err()?;
                    let mut i = 1;
                    for mut row in rows {
                        let data = row_to_table!(row, lua);
                        query_data.set(i, data)?;
                        i += 1;
                    }
                    return Ok(query_data);
                }

                let params = params.into_vec();
                let mut new_params: Vec<MysqlValue> = Vec::new();
                for v in params {
                    if let LuaValue::Table(v) = v {
                        for pairs in v.pairs::<LuaValue, LuaValue>() {
                            let (_, val) = pairs?;
                            new_params.push(lua_value_to_mysql_value(val));
                        }
                    } else {
                        new_params.push(lua_value_to_mysql_value(v));
                    }
                }
                let rows: Vec<Row> = conn.exec(sql, new_params).to_lua_err()?;

                let query_data = lua.create_table()?;
                let mut i = 1;
                for mut row in rows {
                    let data = row_to_table!(row, lua);
                    query_data.set(i, data)?;
                    i += 1;
                }
                Ok(query_data)
            },
        );
        _methods.add_method(
            "exec_first",
            |lua, this, (sql, params): (String, LuaMultiValue)| {
                let mut conn = this.0.get_conn().to_lua_err()?;
                if params.is_empty() {
                    let row: Option<Row> = conn.exec_first(sql, ()).to_lua_err()?;
                    if let Some(mut row) = row {
                        let data = row_to_table!(row, lua);
                        return Ok(data);
                    } else {
                        return lua.create_table();
                    }
                }
                let params = params.into_vec();
                let mut new_params: Vec<MysqlValue> = Vec::new();
                for v in params {
                    if let LuaValue::Table(v) = v {
                        for pairs in v.pairs::<LuaValue, LuaValue>() {
                            let (_, val) = pairs?;
                            new_params.push(lua_value_to_mysql_value(val));
                        }
                    } else {
                        new_params.push(lua_value_to_mysql_value(v));
                    }
                }
                let row: Option<Row> = conn.exec_first(sql, new_params).to_lua_err()?;

                if let Some(mut row) = row {
                    let data = row_to_table!(row, lua);
                    Ok(data)
                } else {
                    lua.create_table()
                }
            },
        );
        _methods.add_method(
            "exec_drop",
            |_, this, (sql, params): (String, LuaMultiValue)| {
                let mut conn = this.0.get_conn().to_lua_err()?;
                if params.is_empty() {
                    conn.exec_drop(sql, ()).to_lua_err()?;
                    return Ok(());
                }
                let params = params.into_vec();
                let mut new_params: Vec<MysqlValue> = Vec::new();
                for v in params {
                    if let LuaValue::Table(v) = v {
                        for pairs in v.pairs::<LuaValue, LuaValue>() {
                            let (_, val) = pairs?;
                            new_params.push(lua_value_to_mysql_value(val));
                        }
                    } else {
                        new_params.push(lua_value_to_mysql_value(v));
                    }
                }

                conn.exec_drop(sql, new_params).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method(
            "exec_batch",
            |_, this, (sql, params): (String, LuaMultiValue)| {
                let mut conn = this.0.get_conn().to_lua_err()?;
                if params.is_empty() {
                    Err(LuaError::ExternalError(Arc::new(WebError::new(
                        6011,
                        "Parameter cannot be empty",
                    ))))
                } else {
                    let params = params.into_vec();
                    let mut new_params: Vec<Vec<MysqlValue>> = Vec::new();
                    for v in params {
                        if let LuaValue::Table(v) = v {
                            let mut other_params: Vec<MysqlValue> = Vec::new();
                            for pair in v.pairs::<LuaValue, LuaValue>() {
                                let (_, tab) = pair?;
                                if let LuaValue::Table(t) = tab {
                                    let mut o_params: Vec<MysqlValue> = Vec::new();
                                    for pairs in t.pairs::<LuaValue, LuaValue>() {
                                        let (_, val) = pairs?;
                                        o_params.push(lua_value_to_mysql_value(val));
                                    }
                                    new_params.push(o_params);
                                } else {
                                    other_params.push(lua_value_to_mysql_value(tab));
                                }
                            }
                            if !other_params.is_empty() {
                                new_params.push(other_params);
                            }
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6012,
                                "Parameter error",
                            ))));
                        }
                    }
                    conn.exec_batch(sql, new_params).to_lua_err()?;
                    Ok(())
                }
            },
        )
    }
}

fn mysql_value_to_lua_value(val: MysqlValue, lua: &Lua) -> LuaResult<LuaValue> {
    match val {
        MysqlValue::NULL => {
            let data = lua.create_string("");
            match data {
                Ok(val) => Ok(LuaValue::String(val)),
                Err(_) => Ok(LuaValue::Nil),
            }
        }
        MysqlValue::Bytes(v) => {
            // let data = String::from_utf8(v).unwrap_or_else(|_| String::from(""));
            let data = lua.create_string(&v);
            match data {
                Ok(val) => Ok(LuaValue::String(val)),
                Err(_) => Ok(LuaValue::Nil),
            }
        }
        MysqlValue::Int(v) => Ok(LuaValue::Integer(v)),
        MysqlValue::UInt(v) => Ok(LuaValue::String(lua.create_string(&v.to_string())?)),
        MysqlValue::Float(v) => Ok(LuaValue::Number(v as f64)),
        MysqlValue::Double(v) => Ok(LuaValue::Number(v)),
        MysqlValue::Date(y, m, d, h, min, s, _) => {
            let format: LuaValue = lua.globals().get("DATEFORMAT")?;

            match format {
                LuaValue::Nil => {
                    let date = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, min, s);
                    let datetime = date.parse::<DateTimeUtc>();
                    match datetime {
                        Ok(val) => Ok(LuaValue::Integer(val.0.timestamp())),
                        Err(_) => Ok(LuaValue::Nil),
                    }
                }
                LuaValue::String(v) => {
                    let ty = v.to_str()?;
                    if ty == "table" {
                        let temp = lua.create_table()?;
                        temp.set("year", y)?;
                        temp.set("month", m)?;
                        temp.set("day", d)?;
                        temp.set("hour", h)?;
                        temp.set("min", min)?;
                        temp.set("sec", s)?;
                        Ok(LuaValue::Table(temp))
                    } else if ty == "timestamp" {
                        let date = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, min, s);
                        let datetime = date.parse::<DateTimeUtc>();
                        match datetime {
                            Ok(val) => Ok(LuaValue::Integer(val.0.timestamp())),
                            Err(_) => Ok(LuaValue::Nil),
                        }
                    } else if ty == "string" {
                        let date = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, min, s);
                        let data = lua.create_string(&date);
                        match data {
                            Ok(val) => Ok(LuaValue::String(val)),
                            Err(_) => Ok(LuaValue::Nil),
                        }
                    } else {
                        let date = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, min, s);
                        let datetime = date.parse::<DateTimeUtc>();
                        match datetime {
                            Ok(val) => Ok(LuaValue::Integer(val.0.timestamp())),
                            Err(_) => Ok(LuaValue::Nil),
                        }
                    }
                }
                _ => {
                    let date = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, min, s);
                    let datetime = date.parse::<DateTimeUtc>();
                    match datetime {
                        Ok(val) => Ok(LuaValue::Integer(val.0.timestamp())),
                        Err(_) => Ok(LuaValue::Nil),
                    }
                }
            }
        }
        MysqlValue::Time(_, _, h, m, s, _) => {
            let format: LuaValue = lua.globals().get("DATEFORMAT")?;

            match format {
                LuaValue::Nil => {
                    let time = format!("{:02}:{:02}:{:02}", h, m, s);
                    let data = lua.create_string(&time);
                    match data {
                        Ok(val) => Ok(LuaValue::String(val)),
                        Err(_) => Ok(LuaValue::Nil),
                    }
                }
                LuaValue::String(v) => {
                    let s = v.to_str()?;
                    if s == "table" {
                        let temp = lua.create_table()?;
                        temp.set("hour", h)?;
                        temp.set("min", m)?;
                        temp.set("sec", s)?;
                        Ok(LuaValue::Table(temp))
                    } else {
                        let time = format!("{:02}:{:02}:{:02}", h, m, s);
                        let data = lua.create_string(&time);
                        match data {
                            Ok(val) => Ok(LuaValue::String(val)),
                            Err(_) => Ok(LuaValue::Nil),
                        }
                    }
                }
                _ => {
                    let time = format!("{:02}:{:02}:{:02}", h, m, s);
                    let data = lua.create_string(&time);
                    match data {
                        Ok(val) => Ok(LuaValue::String(val)),
                        Err(_) => Ok(LuaValue::Nil),
                    }
                }
            }
        }
    }
}

fn lua_value_to_mysql_value(val: LuaValue) -> MysqlValue {
    match val {
        LuaValue::Nil => MysqlValue::NULL,
        LuaValue::Boolean(v) => MysqlValue::from(v),
        LuaValue::LightUserData(_) => MysqlValue::NULL,
        LuaValue::Integer(v) => MysqlValue::Int(v),
        LuaValue::Number(v) => MysqlValue::Double(v),
        LuaValue::String(v) => {
            let data = v.as_bytes();
            let data = Vec::from(data);
            MysqlValue::Bytes(data)
        }
        LuaValue::Table(_) => MysqlValue::NULL,
        LuaValue::Function(_) => MysqlValue::NULL,
        LuaValue::Thread(_) => MysqlValue::NULL,
        LuaValue::UserData(_) => MysqlValue::NULL,
        _ => MysqlValue::NULL,
    }
}

#[mlua::lua_module]
fn mysql(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    lua.create_proxy::<MysqlPool>()
}
