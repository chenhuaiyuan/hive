use mlua::prelude::*;

pub fn create_table_to_json_string<'a>(lua: &'a Lua) -> LuaResult<LuaFunction> {
    let json_func = lua.create_function(|_, table: LuaTable| {
        let json = serde_json::to_string(&table).to_lua_err()?;
        Ok(json)
    });
    json_func
}
