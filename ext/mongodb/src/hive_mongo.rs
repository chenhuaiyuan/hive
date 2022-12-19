use bson::Document;
use hive_time::TimeDuration;
use std::collections::HashMap;

use crate::{
    hive_bson::bson_value_to_lua_value,
    hive_document::{table_to_document, BsonDocument},
    hive_mongo_client_session::MongoClientSession,
    hive_mongo_options::{
        self, MongoAggregateOptions, MongoChangeStreamOptions, MongoCountOptions,
        MongoCreateCollectionOptions, MongoDropCollectionOptions,
        MongoEstimatedDocumentCountOptions, MongoIndexOptions, MongoSelectionCriteria,
    },
};
use mlua::prelude::*;
use mongodb::{event::command, Client, Database, IndexModel, Namespace};

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
            "drop",
            |_, this, options: Option<LuaAnyUserData>| async move {
                if let Some(opt) = options {
                    let options = opt.take::<MongoDropCollectionOptions>()?;
                    this.0.drop(Some(options.0)).await.to_lua_err()?;
                } else {
                    this.0.drop(None).await.to_lua_err()?;
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
        _methods.add_async_method(
            "watch",
            |lua, this, (pipeline, options): (Vec<LuaTable>, Option<LuaAnyUserData>)| async move {
                let mut docs: Vec<Document> = Vec::new();
                for tab in pipeline {
                    let doc = table_to_document(tab)?;
                    docs.push(doc);
                }
                let options = options.map(|v| v.take::<MongoChangeStreamOptions>()?.0);
                let data = this.0.watch(docs, options).await.to_lua_err()?;
                let resp: Vec<MongoChangeStreamEvent<Document>> = Vec::new();
                while let Some(event) = data.next_if_any().await? {
                    resp.push(event);
                }
                Ok(resp)
            },
        );
        _methods.add_async_method(
            "watch_with_session",
            |lua,
             this,
             (pipeline, session, options): (
                Vec<LuaTable>,
                LuaAnyUserData,
                Option<LuaAnyUserData>,
            )| async move {
                let mut docs: Vec<Document> = Vec::new();
                for tab in pipeline {
                    let doc = table_to_document(tab)?;
                    docs.push(doc);
                }
                let options = options.map(|v| v.take::<MongoChangeStreamOptions>()?.0);
                let session = session.borrow_mut::<MongoClientSession>()?.0;
                let data = this
                    .0
                    .watch_with_session(docs, options, session)
                    .await
                    .to_lua_err()?;
                let resp: Vec<MongoChangeStreamEvent<Document>> = Vec::new();
                while let Some(event) = data.next(session).await? {
                    resp.push(event);
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
        _methods.add_async_method(
            "drop",
            |_, this, options: Option<LuaAnyUserData>| async move {
                if let Some(opt) = options {
                    let options = opt.take::<MongoDropCollectionOptions>()?;
                    this.0.drop(Some(options.0)).await.to_lua_err()?;
                } else {
                    this.0.drop(None).await.to_lua_err()?;
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
        _methods.add_async_method(
            "estimated_document_count",
            |_, this, options: Option<LuaAnyUserData>| async move {
                let options = options.map(|v| v.take::<MongoEstimatedDocumentCountOptions>()?.0);
                let data = this
                    .0
                    .estimated_document_count(options)
                    .await
                    .to_lua_err()?;
                Ok(data)
            },
        );
        _methods.add_async_method(
            "count_documents",
            |_, this, (filter, options): (Option<LuaTable>, Option<LuaAnyUserData>)| async move {
                let filter = filter.map(|v| table_to_document(v)?);
                let options = options.map(|v| v.take::<MongoCountOptions>()?.0);
                let data = this.0.count_documents(filter, options).await.to_lua_err()?;
                Ok(data)
            },
        );
        _methods.add_async_method(
            "count_documents_with_session",
            |_,
             this,
             (session, filter, optionsm): (
                LuaAnyUserData,
                Option<LuaTable>,
                Option<LuaAnyUserData>,
            )| async move {
                let filter = filter.map(|v| table_to_document(v)?);
                let options = options.map(|v| v.take::<MongoCountOptions>()?.0);
                let session = session.borrow_mut::<MongoClientSession>()?.0;
                let data = this
                    .0
                    .count_documents_with_session(filter, options, session)
                    .await
                    .to_lua_err()?;
                Ok(data)
            },
        );
        // _methods.add_async_method("create_index", |_, this, (index, options): (LuaAnyUserData, Option<LuaAnyUserData>)| async move {
        //     let index = index.take::<MongoIndexModel>()?.0;
        //     let options = options.map(|v| v.take::<>())
        // });
    }
}

pub struct MongoNamespace(Namespace);

impl LuaUserData for MongoNamespace {}

pub struct MongoIndexModel(IndexModel);

pub fn create_index_model(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, (keys, options): (LuaTable, Option<LuaAnyUserData>)| {
        let keys = table_to_document(keys)?;
        let options = options.map(|v| v.take::<MongoIndexOptions>()?.0);
        let opt = IndexModel::builder().keys(keys).options(options).build();
        Ok(opt)
    })
}

impl LuaUserData for MongoIndexModel {}
