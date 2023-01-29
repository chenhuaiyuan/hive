use mlua::prelude::*;
use rusqlite::{params_from_iter, Connection};
use rusqlite::{types::Value as SqliteValue, OpenFlags};

use crate::{flags::SqliteOpenFlags, sqlite_value_to_lua_value, table_to_params};

pub struct SqliteConnection(Connection);

impl LuaUserData for SqliteConnection {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("open", |_, path: String| {
            let conn = Connection::open(path).to_lua_err()?;
            Ok(SqliteConnection(conn))
        });
        _methods.add_function("open_in_memory", |_, ()| {
            let conn = Connection::open_in_memory().to_lua_err()?;
            Ok(SqliteConnection(conn))
        });
        _methods.add_function("open_with_flags", |_, (path, flags): (String, LuaTable)| {
            let mut f = OpenFlags::empty();

            for pair in flags.pairs::<LuaValue, LuaAnyUserData>() {
                let (_, flag) = pair?;
                let flag = flag.take::<SqliteOpenFlags>()?;
                f = f | flag.0;
            }
            let conn = Connection::open_with_flags(path, f).to_lua_err()?;
            Ok(SqliteConnection(conn))
        });
        _methods.add_function("open_in_memory_with_flags", |_, flags: LuaTable| {
            let mut f = OpenFlags::empty();

            for pair in flags.pairs::<LuaValue, LuaAnyUserData>() {
                let (_, flag) = pair?;
                let flag = flag.take::<SqliteOpenFlags>()?;
                f = f | flag.0;
            }
            let conn = Connection::open_in_memory_with_flags(f).to_lua_err()?;
            Ok(SqliteConnection(conn))
        });
        _methods.add_method("execute_batch", |_, this, sql: String| {
            this.0.execute_batch(&sql).to_lua_err()?;
            Ok(())
        });
        _methods.add_method("execute", |_, this, (sql, params): (String, LuaTable)| {
            let params = table_to_params(params)?;
            let params = params_from_iter(params);
            let resp = this.0.execute(&sql, params).to_lua_err()?;
            Ok(resp)
        });
        _methods.add_method_mut(
            "query",
            |lua, this, (sql, params, query_field): (String, LuaTable, LuaTable)| {
                let params = table_to_params(params)?;
                let params = params_from_iter(params);
                let mut stmt = this.0.prepare(&sql).to_lua_err()?;
                let mut rows = stmt.query(params).to_lua_err()?;
                let table = lua.create_table()?;
                let mut idx = 1;
                while let Some(row) = rows.next().to_lua_err()? {
                    let sub_table = lua.create_table()?;
                    for pair in query_field.clone().pairs::<LuaValue, String>() {
                        let (_, field) = pair?;
                        let data: SqliteValue = row.get(field.as_str()).to_lua_err()?;
                        let data = sqlite_value_to_lua_value(lua, data)?;
                        sub_table.set(field, data)?;
                    }
                    table.set(idx, sub_table)?;
                    idx += 1;
                }
                Ok(table)
            },
        );
        _methods.add_method_mut(
            "query_first",
            |lua, this, (sql, params, query_field): (String, LuaTable, LuaTable)| {
                let params = table_to_params(params)?;
                let params = params_from_iter(params);
                let mut stmt = this.0.prepare(&sql).to_lua_err()?;
                let table = stmt
                    .query_row(params, |row| {
                        let table = lua.create_table().unwrap();
                        for pair in query_field.pairs::<LuaValue, String>() {
                            let (_, field) = pair.unwrap();
                            let data: SqliteValue = row.get(field.as_str())?;
                            let data = sqlite_value_to_lua_value(lua, data).unwrap();
                            table.set(field, data).unwrap();
                        }
                        Ok(table)
                    })
                    .to_lua_err()?;

                Ok(table)
            },
        );
        _methods.add_method_mut("insert", |_, this, (sql, params): (String, LuaTable)| {
            let params = table_to_params(params)?;
            let params = params_from_iter(params);
            let mut stmt = this.0.prepare(&sql).to_lua_err()?;
            let data = stmt.insert(params).to_lua_err()?;
            Ok(data)
        });
    }
}
