use hive_time::TimeDuration;
use std::collections::HashMap;

use crate::{
    hive_mongo_client_session::MongoClientSession,
    hive_mongo_options::{self, MongoCreateCollectionOptions, MongoDropCollectionOptions},
};
use mlua::prelude::*;
use mongodb::{Client, Database, Namespace};

pub struct MongoClient(Client);

impl LuaUserData for MongoClient {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_async_function("with_uri_str", |_, uri: String| async move {
            let client = Client::with_uri_str(uri).await.to_lua_err()?;
            Ok(MongoClient(client))
        });
        // _methods.add_function("with_options", ||);
        // _methods.add_method("selection_criteria", ||);
        _methods.add_method("read_concern", |lua, this, ()| {
            let read_concern = this.0.read_concern();
            if let Some(rc) = read_concern {
                Ok(LuaValue::UserData(lua.create_userdata(
                    hive_mongo_options::MongoReadConcern(rc),
                )?))
            } else {
                Ok(LuaValue::Nil)
            }
        });
        _methods.add_method("write_concern", |lua, this, ()| {
            let write_concern = this.0.write_concern();
            if let Some(wc) = write_concern {
                Ok(LuaValue::UserData(lua.create_userdata(
                    hive_mongo_options::MongoWriteConcern(write_concern),
                )?))
            } else {
                Ok(LuaValue::Nil)
            }
        });
        _methods.add_method("database", |_, this, name: String| {
            Ok(MongoDatabase(this.0.database(&name)))
        });
    }
}

pub struct MongoDatabase(Database);

impl LuaUserData for MongoDatabase {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method("name", |lua, this, ()| lua.create_string(this.0.name()));
        // _methods.add_method("selection_criteria", |lua, |)
        _methods.add_method("read_concern", |lua, this, ()| {
            let read_concern = this.0.read_concern();
            if let Some(rc) = read_concern {
                Ok(LuaValue::UserData(lua.create_userdata(
                    hive_mongo_options::MongoReadConcern(rc),
                )?))
            } else {
                Ok(LuaValue::Nil)
            }
        });
        _methods.add_method("write_concern", |lua, this, ()| {
            let write_concern = this.0.write_concern();
            if let Some(wc) = write_concern {
                Ok(LuaValue::UserData(lua.create_userdata(
                    hive_mongo_options::MongoWriteConcern(write_concern),
                )?))
            } else {
                Ok(LuaValue::Nil)
            }
        });
        _methods.add_method("collection", |lua, name: String| {});
    }
}

pub struct MongoCollection(Collection);

impl LuaUserData for MongoCollection {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method("name", |lua, this, ()| {
            let name = this.0.name();
            lua.create_string(name)
        });
        _methods.add_method("namespace", |lua, this, ()| {
            let namespace = this.0.namespace();
            lua.create_userdata(MongoNamespace(namespace))
        });
        // TODO
        // _methods.add_method("selection_criteria", ||)
        _methods.add_method("read_concern", |lua, this, ()| {
            let read_concern = this.0.read_concern();
            if let Some(rc) = read_concern {
                Ok(LuaValue::UserData(lua.create_userdata(
                    hive_mongo_options::MongoReadConcern(rc),
                )?))
            } else {
                Ok(LuaValue::Nil)
            }
        });
        _methods.add_method("write_concern", |lua, this, ()| {
            let write_concern = this.0.write_concern();
            if let Some(wc) = write_concern {
                Ok(LuaValue::UserData(lua.create_userdata(
                    hive_mongo_options::MongoWriteConcern(write_concern),
                )?))
            } else {
                Ok(LuaValue::Nil)
            }
        });
        _methods.add_method("close", |_, this, options: Option<LuaAnyUserData>| {
            if let Some(opt) = options {
                let options = opt.take::<MongoDropCollectionOptions>()?;
                this.0.drop(Some(options.0))?;
            } else {
                this.0.drop(None)?;
            }
            Ok(())
        });
        _methods.add_method(
            "drop_with_session",
            |_, this, (session, options): (LuaAnyUserData, Option<LuaAnyUserData>)| {
                let options = options.map(|v| v.take::<MongoDropCollectionOptions>()?.0);
                let session = session.take::<MongoClientSession>()?;
                this.0.drop_with.session(options, session.0).to_lua_err()?;
                Ok(())
            },
        );
        // TODO
        // list_collection以后实现
        _methods.add_method(
            "create_collection",
            |_, this, (name, options): (String, Option<LuaAnyUserData>)| {
                let options = options.map(|v| v.take::<MongoCreateCollectionOptions>()?.0);
                this.0.create_collection(&name, options).to_lua_err()?;
                Ok(())
            },
        );
    }
}

pub struct MongoNamespace(Namespace);

impl LuaUserData for MongoNamespace {}
