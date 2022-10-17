use std::{collections::HashMap, time::Duration};

use mlua::prelude::*;
use serde_json::Value as JsonValue;
use ureq::{Request, Response};

#[derive(Clone)]
pub struct Http(Request);

impl Http {
    pub fn new(req: Request) -> Self {
        Self(req)
    }
}

impl LuaUserData for Http {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("timeout", |_, (this, secs): (LuaAnyUserData, u64)| {
            let this = this.take::<Self>()?;
            let r = this.0.timeout(Duration::from_secs(secs));
            Ok(Http::new(r))
        });
        _methods.add_function(
            "setHeader",
            |_, (this, header): (LuaAnyUserData, LuaMultiValue)| {
                let this = this.take::<Self>()?;
                let header = header.into_vec();
                let head;
                let val;

                if header.len() >= 2 {
                    if let LuaValue::String(v) = &header[0] {
                        head = v.to_str()?.to_string();
                    } else {
                        head = "".to_string();
                    }
                    if let LuaValue::String(v) = &header[1] {
                        val = v.to_str()?.to_string();
                    } else {
                        val = "".to_string();
                    }
                } else {
                    head = "".to_string();
                    val = "".to_string();
                }

                let r = if !head.is_empty() && !val.is_empty() {
                    this.0.set(&head, &val)
                } else {
                    this.0
                };
                Ok(Http::new(r))
            },
        );
        _methods.add_function("get", |_, url: String| {
            println!("{}", url);
            let req = ureq::get(&url);
            Ok(Http::new(req))
        });
        _methods.add_function("post", |_, url: String| {
            let req = ureq::post(&url);
            Ok(Http::new(req))
        });
        _methods.add_function("head", |_, url: String| {
            let req = ureq::head(&url);
            Ok(Http::new(req))
        });
        _methods.add_function("patch", |_, url: String| {
            let req = ureq::patch(&url);
            Ok(Http::new(req))
        });
        _methods.add_function("put", |_, url: String| {
            let req = ureq::put(&url);
            Ok(Http::new(req))
        });
        _methods.add_function("sendJson", |_, (this, data): (LuaAnyUserData, LuaTable)| {
            let this = this.take::<Self>()?;
            let resp = this.0.send_json(data).to_lua_err()?;
            Ok(LuaResponse::new(resp))
        });
        _methods.add_function("sendString", |_, (this, data): (LuaAnyUserData, String)| {
            let this = this.take::<Self>()?;
            let resp = this.0.send_string(&data).to_lua_err()?;
            Ok(LuaResponse::new(resp))
        });
        _methods.add_function("sendForm", |_, (this, data): (LuaAnyUserData, LuaTable)| {
            let this = this.take::<Self>()?;
            let data = table_to_vec(data)?;
            let mut form_data: Vec<(&str, &str)> = Vec::new();
            for (key, val) in data.iter() {
                form_data.push((key, val));
            }
            let form = form_data.as_slice();
            let resp = this.0.send_form(form).to_lua_err()?;
            Ok(LuaResponse::new(resp))
        });
        _methods.add_function("query", |_, (this, data): (LuaAnyUserData, LuaTable)| {
            let this = this.take::<Self>()?;
            let data = table_to_vec(data)?;
            let mut req = this.0;
            for (key, val) in data {
                req = req.query(&key, &val);
            }
            let resp = req.call().to_lua_err()?;
            Ok(LuaResponse::new(resp))
        });
        _methods.add_function("call", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let resp = this.0.call().to_lua_err()?;
            Ok(LuaResponse::new(resp))
        });
    }
}

pub struct LuaResponse(Response);

impl LuaResponse {
    pub fn new(resp: Response) -> Self {
        Self(resp)
    }
}

impl LuaUserData for LuaResponse {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method("status", |_, this, ()| {
            Ok(LuaValue::Integer(this.0.status() as i64))
        });
        _methods.add_function("json", |lua, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let json_data = lua.create_table()?;
            let data: HashMap<String, serde_json::Value> = this.0.into_json().to_lua_err()?;
            for (key, val) in data {
                json_data.set(key, json_value_to_lua_value(val, lua)?)?;
            }
            Ok(json_data)
        });
        _methods.add_function("text", |lua, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let data = this.0.into_string().to_lua_err()?;
            Ok(lua.create_string(&data))
        });
    }
}

fn table_to_vec(val: LuaTable) -> LuaResult<Vec<(String, String)>> {
    let mut data: Vec<(String, String)> = Vec::new();
    for pair in val.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;
        if let LuaValue::String(v) = key {
            let k = v.to_str()?.to_string();

            match value {
                LuaValue::Boolean(v) => {
                    if v {
                        data.push((k, "true".to_string()));
                    } else {
                        data.push((k, "false".to_string()));
                    }
                }
                LuaValue::Integer(v) => {
                    data.push((k, v.to_string()));
                }
                LuaValue::Number(v) => {
                    data.push((k, v.to_string()));
                }
                LuaValue::String(v) => {
                    let val = v.to_str()?.to_string();
                    data.push((k, val));
                }
                _ => {}
            }
        }
    }
    Ok(data)
}

fn json_value_to_lua_value(val: JsonValue, lua: &Lua) -> LuaResult<LuaValue> {
    match val {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(v) => Ok(LuaValue::Boolean(v)),
        JsonValue::Number(v) => {
            if v.is_i64() {
                let num = v.as_i64();
                if let Some(num) = num {
                    return Ok(LuaValue::Integer(num));
                }
            } else if v.is_u64() {
                let num = v.as_u64();
                if let Some(num) = num {
                    return Ok(LuaValue::Number(num as f64));
                }
            } else if v.is_f64() {
                let num = v.as_f64();
                if let Some(num) = num {
                    return Ok(LuaValue::Number(num));
                }
            }
            return Ok(LuaValue::Integer(0));
        }
        JsonValue::String(v) => {
            let s = lua.create_string(&v)?;
            return Ok(LuaValue::String(s));
        }
        _ => Ok(LuaValue::Nil),
    }
}