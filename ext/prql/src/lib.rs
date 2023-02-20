use mlua::prelude::*;
use prql_compiler::{compile, sql::Dialect, Options, Target};

fn create_dialect_table(lua: &Lua) -> LuaResult<LuaTable> {
    lua.create_table_from([
        ("ansi", PrqlDialect(Dialect::Ansi)),
        ("bigquery", PrqlDialect(Dialect::BigQuery)),
        ("clickhouse", PrqlDialect(Dialect::ClickHouse)),
        ("duckdb", PrqlDialect(Dialect::DuckDb)),
        ("generic", PrqlDialect(Dialect::Generic)),
        ("hive", PrqlDialect(Dialect::Hive)),
        ("mssql", PrqlDialect(Dialect::MsSql)),
        ("mysql", PrqlDialect(Dialect::MySql)),
        ("postgres", PrqlDialect(Dialect::PostgreSql)),
        ("sqlite", PrqlDialect(Dialect::SQLite)),
        ("snowflake", PrqlDialect(Dialect::Snowflake)),
    ])
}

pub struct PrqlDialect(Dialect);

impl LuaUserData for PrqlDialect {}

pub struct PrqlOptions(Options);

impl LuaUserData for PrqlOptions {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function(
            "new",
            |_, (format, dialect, signature_comment): (bool, LuaAnyUserData, bool)| {
                let dialect = dialect.take::<PrqlDialect>()?;
                let options = Options {
                    format,
                    target: Target::Sql(Some(dialect.0)),
                    signature_comment,
                };
                Ok(PrqlOptions(options))
            },
        );
    }
}

fn create_compile(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, (prql, options): (String, LuaAnyUserData)| {
        let options = options.borrow::<PrqlOptions>()?;
        let sql_result = compile(&prql, options.0.clone());
        match sql_result {
            Ok(sql) => Ok(sql),
            Err(err_message) => Err(LuaError::RuntimeError(err_message.to_string())),
        }
    })
}

#[mlua::lua_module]
fn prql(lua: &Lua) -> LuaResult<LuaTable> {
    lua.create_table_from([
        ("dialect", LuaValue::Table(create_dialect_table(lua)?)),
        (
            "options",
            LuaValue::UserData(lua.create_proxy::<PrqlOptions>()?),
        ),
        ("compile", LuaValue::Function(create_compile(lua)?)),
    ])
}
