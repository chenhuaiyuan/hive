use dateparser::DateTimeUtc;
use mlua::prelude::*;
use mysql_async::{prelude::Queryable, Opts, Pool, Row, Value as MysqlValue};

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
            |_, (username, password, address): (String, String, String)| {
                let mysql_url = format!("mysql://{}:{}@{}", username, password, address);
                let opts = Opts::from_url(&mysql_url).to_lua_err()?;
                let pool = Pool::new(opts);
                Ok(MysqlPool(pool))
            },
        );
        _methods.add_async_method("query", |lua, this, sql: String| async move {
            let query_data = lua.create_table()?;

            let mut conn = this.0.get_conn().await.to_lua_err()?;

            let rows: Vec<Row> = conn.query(sql).await.to_lua_err()?;
            let mut i = 1;
            for mut row in rows {
                let data = row_to_table!(row, lua);
                query_data.set(i, data)?;
                i += 1;
            }

            Ok(query_data)
        });
        _methods.add_async_method("query_first", |lua, this, sql: String| async move {
            let mut conn = this.0.get_conn().await.to_lua_err()?;

            let row: Option<Row> = conn.query_first(sql).await.to_lua_err()?;
            if let Some(mut row) = row {
                let data = row_to_table!(row, lua);
                Ok(data)
            } else {
                lua.create_table()
            }
        });
        _methods.add_async_method(
            "exec",
            |lua, this, (sql, params): (String, LuaMultiValue)| async move {
                let mut conn = this.0.get_conn().await.to_lua_err()?;
                if params.is_empty() {
                    let query_data = lua.create_table()?;
                    let rows: Vec<Row> = conn.query(sql).await.to_lua_err()?;
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
                let rows: Vec<Row> = conn.exec(sql, new_params).await.to_lua_err()?;

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
        _methods.add_async_method(
            "exec_first",
            |lua, this, (sql, params): (String, LuaMultiValue)| async move {
                let mut conn = this.0.get_conn().await.to_lua_err()?;
                if params.is_empty() {
                    let row: Option<Row> = conn.query_first(sql).await.to_lua_err()?;
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
                let row: Option<Row> = conn.exec_first(sql, new_params).await.to_lua_err()?;

                if let Some(mut row) = row {
                    let data = row_to_table!(row, lua);
                    Ok(data)
                } else {
                    lua.create_table()
                }
            },
        );
        _methods.add_async_method(
            "exec_drop",
            |_, this, (sql, params): (String, LuaMultiValue)| async move {
                let mut conn = this.0.get_conn().await.to_lua_err()?;
                if params.is_empty() {
                    conn.query_drop(sql).await.to_lua_err()?;
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

                conn.exec_drop(sql, new_params).await.to_lua_err()?;
                Ok(())
            },
        )
    }
}

fn mysql_value_to_lua_value(val: mysql_async::Value, lua: &Lua) -> LuaResult<LuaValue> {
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
        MysqlValue::UInt(v) => Ok(LuaValue::Number(v as f64)),
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
