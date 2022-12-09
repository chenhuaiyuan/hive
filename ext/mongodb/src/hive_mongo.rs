use bson::Document;
use hive_time::TimeDuration;
use std::collections::HashMap;

use crate::{
    hive_bson::bson_value_to_lua_value,
    hive_document::BsonDocument,
    hive_mongo_client_session::MongoClientSession,
    hive_mongo_options::{
        self, MongoAggregateOptions, MongoCreateCollectionOptions, MongoDropCollectionOptions,
        MongoSelectionCriteria,
    },
};
use mlua::prelude::*;
use mongodb::{event::command, Client, Database, Namespace};

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

        _methods.add_async_method(
            "close",
            |_, this, options: Option<LuaAnyUserData>| async move {
                if let Some(opt) = options {
                    let options = opt.take::<MongoDropCollectionOptions>()?;
                    this.0.drop(Some(options.0)).await?;
                } else {
                    this.0.drop(None).await?;
                }
                Ok(())
            },
        );
        _methods.add_async_method(
            "drop_with_session",
            |_, this, (session, options): (LuaAnyUserData, Option<LuaAnyUserData>)| async move {
                let options = options.map(|v| v.take::<MongoDropCollectionOptions>()?.0);
                let session = session.borrow_mut::<MongoClientSession>()?.0;
                this.0
                    .drop_with_session(options, session)
                    .await
                    .to_lua_err()?;
                Ok(())
            },
        );
        // TODO
        // list_collection以后实现
        _methods.add_async_method(
            "create_collection",
            |_, this, (name, options): (String, Option<LuaAnyUserData>)| async move {
                let options = options.map(|v| v.take::<MongoCreateCollectionOptions>()?.0);
                this.0
                    .create_collection(&name, options)
                    .await
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_async_method("create_collection_with_session", |_, this, (name, options, session): (String, Option<LuaAnyUserData>, LuaAnyUserData)| async move {
            let options = options.map(|v| v.take::<MongoCreateCollectionOptions>()?.0);
            let session = session.borrow_mut::<MongoClientSession>()?.0;
            this.0.create_collection_with_session(&name, options, session).await.to_lua_err();
            Ok(())
        });
        _methods.add_async_method(
            "run_command",
            |lua, this, (command, selection_criteria): (LuaTable, Option<LuaAnyUserData>)| async move {
                let mut doc = Document::new();
                for pairs in v.pairs::<String, LuaValue>() {
                    let (key, val) = pairs?;
                    doc.insert(key, lua_value_to_bson_value(val)?)
                }
                let selection_criteria =
                    selection_criteria.map(|v| v.take::<MongoSelectionCriteria>()?.0);
                let doc = this.0.run_command(doc, selection_criteria).await.to_lua_err()?;
                bson_value_to_lua_value(&lua, doc)
            },
        );
        _methods.add_async_method(
            "run_command_with_session",
            |lua,
             this,
             (command, selection_criteria, session): (
                LuaTable,
                Option<LuaAnyUserData>,
                LuaAnyUserData,
            )| async move {
                let mut doc = Document::new();
                for pairs in v.pairs::<String, LuaValue>() {
                    let (key, val) = pairs?;
                    doc.insert(key, lua_value_to_bson_value(val)?)
                }
                let selection_criteria =
                    selection_criteria.map(|v| v.take::<MongoSelectionCriteria>()?.0);
                let session = session.borrow_mut::<MongoClientSession>()?.0;
                let doc = this
                    .0
                    .run_command_with_session(doc, selection_criteria, session)
                    .await
                    .to_lua_err()?;
                bson_value_to_lua_value(&lua, doc)
            },
        );
        _methods.add_async_method(
            "aggregate",
            |lua, this, (pipeline, options): (Vec<LuaTable>, Option<LuaAnyUserData>)| async move {
                let mut docs: Vec<Document> = Vec::new();
                for tab in pipeline {
                    let mut doc = Document::new();
                    for pairs in tab.pairs::<String, LuaValue>() {
                        let (key, val) = pairs?;
                        doc.insert(key, lua_value_to_bson_value(val)?)
                    }
                    docs.push(doc);
                }
                let options = options.map(|v| v.take::<MongoAggregateOptions>()?.0);
                let data = this.0.aggregate(docs, options).await.to_lua_err()?;
                let mut resp: Vec<LuaTable> = Vec::new();
                while let Some(doc) = data.next().await {
                    let doc = doc.to_lua_err()?;
                    resp.push(bson_value_to_lua_value(&lua, doc)?);
                }
                Ok(resp)
            },
        );
        _methods.add_async_method(
            "aggregate_with_session",
            |lua,
             this,
             (pipeline, options, session): (
                Vec<luaTable>,
                Option<LuaAnyUserData>,
                LuaAnyUserData,
            )| async move {
                let mut docs: Vec<Document> = Vec::new();
                for tab in pipeline {
                    let mut doc = Document::new();
                    for pairs in tab.pairs::<String, LuaValue>() {
                        let (key, val) = pairs?;
                        doc.insert(key, lua_value_to_bson_value(val)?)
                    }
                    docs.push(doc);
                }
                let options = options.map(|v| v.take::<MongoAggregateOptions>()?.0);
                let session = session.borrow_mut::<MongoClientSession>()?.0;
                let data = this
                    .0
                    .aggregate_with_session(docs, options, session)
                    .await
                    .to_lua_err()?;
                let mut resp: Vec<LuaTable> = Vec::new();
                while let Some(doc) = data.next(&mut session).await.transpose()? {
                    resp.push(bson_value_to_lua_value(&lua, doc)?);
                }
                Ok(resp)
            },
        );
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
    }
}

pub struct MongoNamespace(Namespace);

impl LuaUserData for MongoNamespace {}
