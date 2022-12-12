use crate::hive_bson::bson_value_to_lua_value;
use bson::document::{Document, Iter, Keys};
use mlua::prelude::*;

pub struct BsonDocument(Document);

pub fn create_document(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| Ok(BsonDocument(Document::new())))
}

impl LuaUserData for BsonDocument {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method("iter", |lua, this, ()| {
            lua.create_userdata(BsonDocumentIter(this.0.iter()))
        });
        _methods.add_method_mut("clear", |_, this, ()| {
            this.0.clear();
            Ok(())
        });
        _methods.add_method_mut("get", |lua, this, key: String| {
            let data = this.0.get(key);
            if let Some(data) = data {
                bson_value_to_lua_value(&lua, data)
            } else {
                Ok(LuaValue::Nil)
            }
        });
        _methods.add_method("is_null", |_, this, key: String| Ok(this.0.is_null(key)));
        _methods.add_method("contains_key", |_, this, key: String| {
            Ok(this.0.contains_key(key))
        });
        _methods.add_method("keys", |lua, this, ()| {
            let keys = this.0.keys();
            let key_table = lua.create_table()?;
            let mut i = 1;
            for key in keys {
                key_table.set(i, key)?;
            }
            Ok(key_table)
        });
        _methods.add_method("values", |lua, this, ()| {
            let values = this.0.values();
            let val_table = lua.create_table()?;
            let mut i = 1;
            for val in values {
                val_table.set(i, bson_value_to_lua_value(&lua, val)?)?;
            }
            Ok(val_table)
        });
        _methods.add_method("len", |_, this, ()| Ok(this.0.len()));
        _methods.add_method("is_empty", |_, this, ()| Ok(this.0.is_empty()));
    }
}

pub fn document_to_table(lua: &Lua, doc: Document) -> LuaResult<LuaTable> {
    let map = lua.create_table()?;
    for (k, v) in doc.iter() {
        map.set(k.clone(), bson_value_to_lua_value(&lua, v.clone())?)?;
    }
    Ok(map)
}

pub fn table_to_document(table: LuaTable) -> LuaResult<Document> {
    let mut doc = Document::new();
    for pairs in table.pairs::<String, LuaValue>() {
        let (key, val) = pairs?;
        doc.insert(key, lua_value_to_bson_value(val)?)
    }
    Ok(doc)
}

pub struct BsonDocumentIter(Iter);

impl LuaUserData for BsonDocumentIter {}

pub struct DocKeys(Keys);

impl LuaUserData for DocKeys {}
