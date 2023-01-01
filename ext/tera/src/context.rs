use hive_base::{json_value_to_lua_value, lua_value_to_json_value};
use mlua::prelude::*;
use tera::Context;

pub struct TeraContext(Context);

impl LuaUserData for TeraContext {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("new", |_, ()| Ok(TeraContext(Context::new())));
        _methods.add_method_mut("insert", |_, this, (key, val): (String, LuaValue)| {
            let val = lua_value_to_json_value(val)?;
            this.0.insert(key, &val);
            Ok(())
        });
        _methods.add_method_mut("extend", |_, this, source: LuaAnyUserData| {
            let source = source.take::<TeraContext>()?;
            this.0.extend(source.0);
            Ok(())
        });
        _methods.add_function("into_json", |lua, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let json_data = this.0.into_json();
            json_value_to_lua_value(lua, json_data)
        });
        _methods.add_method("get", |lua, this, index: String| {
            let data = this.0.get(&index);
            if let Some(data) = data {
                json_value_to_lua_value(lua, data.clone())
            } else {
                Ok(LuaValue::Nil)
            }
        });
        _methods.add_method_mut("remove", |lua, this, index: String| {
            let data = this.0.remove(&index);
            if let Some(data) = data {
                json_value_to_lua_value(lua, data)
            } else {
                Ok(LuaValue::Nil)
            }
        });
        _methods.add_method("contains_key", |_, this, index: String| {
            Ok(this.0.contains_key(&index))
        });
    }
}

pub fn create_context(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    lua.create_proxy::<TeraContext>()
}
