mod hive_bson;
mod hive_document;
mod hive_mongo;
mod hive_mongo_client_session;
mod hive_mongo_options;
use mlua::prelude::*;

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
