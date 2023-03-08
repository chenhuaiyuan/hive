use mlua::prelude::*;
use serde_json::Value as JsonValue;
use tera::Context;

pub struct TeraContext(Context);

impl LuaUserData for TeraContext {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("new", |_, ()| Ok(TeraContext(Context::new())));
        _methods.add_method_mut("insert", |lua, this, (key, val): (String, LuaValue)| {
            let val: JsonValue = lua.from_value(val)?;
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
            lua.to_value(&json_data)
        });
        _methods.add_method("get", |lua, this, index: String| {
            let data = this.0.get(&index);
            if let Some(data) = data {
                lua.to_value(&data)
            } else {
                Ok(LuaValue::Nil)
            }
        });
        _methods.add_method_mut("remove", |lua, this, index: String| {
            let data = this.0.remove(&index);
            if let Some(data) = data {
                lua.to_value(&data)
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
