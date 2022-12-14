use hive_time::TimeDuration;
use mlua::prelude::*;

use ureq::Request;

use crate::{lua_value_to_json_value, response::ReqResponse};

pub struct ReqRequest(pub Request);

impl LuaUserData for ReqRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function(
            "timeout",
            |_, (this, timeout): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let timeout = timeout.take::<TimeDuration>()?;
                Ok(ReqRequest(this.0.timeout(timeout.0)))
            },
        );
        _methods.add_function("call", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let data = this.0.call().to_lua_err()?;
            Ok(ReqResponse(data))
        });
        _methods.add_function(
            "send_json",
            |_, (this, data): (LuaAnyUserData, LuaTable)| {
                let this = this.take::<Self>()?;
                let data = lua_value_to_json_value(mlua::Value::Table(data))?;
                let resp = this.0.send_json(data).to_lua_err()?;
                Ok(ReqResponse(resp))
            },
        );
        _methods.add_function(
            "send_bytes",
            |_, (this, data): (LuaAnyUserData, Vec<u8>)| {
                let this = this.take::<Self>()?;
                let resp = this.0.send_bytes(&data).to_lua_err()?;
                Ok(ReqResponse(resp))
            },
        );
        _methods.add_function(
            "send_string",
            |_, (this, data): (LuaAnyUserData, String)| {
                let this = this.take::<Self>()?;
                let resp = this.0.send_string(&data).to_lua_err()?;
                Ok(ReqResponse(resp))
            },
        );
        _methods.add_function(
            "send_form",
            |_, (this, data): (LuaAnyUserData, LuaTable)| {
                let this = this.take::<Self>()?;
                let mut form: Vec<(String, String)> = Vec::new();
                for pairs in data.pairs::<String, String>() {
                    let (key, val) = pairs?;
                    form.push((key, val));
                }
                let form = form
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect::<Vec<(&str, &str)>>();
                let resp = this.0.send_form(&form).to_lua_err()?;
                Ok(ReqResponse(resp))
            },
        );
        // todo
        // _methods.add_function("send", function)
        _methods.add_function(
            "set",
            |_, (this, header, value): (LuaAnyUserData, String, String)| {
                let this = this.take::<Self>()?;
                Ok(ReqRequest(this.0.set(&header, &value)))
            },
        );
        _methods.add_method("header", |_, this, name: String| {
            let data = this.0.header(&name);
            let data = data.map(|v| v.to_owned());
            Ok(data)
        });
        _methods.add_method("header_names", |_, this, ()| Ok(this.0.header_names()));
        _methods.add_method("has", |_, this, name: String| Ok(this.0.has(&name)));
        _methods.add_method("all", |_, this, name: String| {
            let data = this.0.all(&name);
            let data = data.iter().map(|&v| v.to_owned()).collect::<Vec<String>>();
            Ok(data)
        });
        _methods.add_function(
            "query",
            |_, (this, param, value): (LuaAnyUserData, String, String)| {
                let this = this.take::<Self>()?;
                Ok(ReqRequest(this.0.query(&param, &value)))
            },
        );
        _methods.add_method("method", |_, this, ()| Ok(this.0.method().to_owned()));
        _methods.add_method("url", |_, this, ()| Ok(this.0.url().to_owned()));
        // todo
        // _methods.add_method("request_url", method)
    }
}
