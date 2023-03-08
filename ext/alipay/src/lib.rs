mod error;

use alipay_rs::{Cli, Client, ClientWithParams, MutCli, Response as AlipayResponse};
use error::Error as WebError;
use mlua::prelude::*;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AlipayClient(Client);

pub struct AlipayClientWithParams(ClientWithParams);

macro_rules! table_to_map {
    ($lua: ident, $table: expr) => {{
        let mut data = HashMap::new();
        for pair in $table.pairs::<String, LuaValue>() {
            let (key, val) = pair?;
            let json_value: JsonValue = $lua.from_value(val)?;
            data.insert(key, json_value);
        }
        data
    }};
}

impl LuaUserData for AlipayClient {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function(
            "new",
            |_,
             (app_id, public_key, private_key, sandbox, app_cert_sn, alipay_root_cert_sn): (
                String,
                String,
                String,
                bool,
                Option<String>,
                Option<String>,
            )| {
                Ok(AlipayClient(Client::new(
                    app_id,
                    public_key,
                    private_key,
                    app_cert_sn,
                    alipay_root_cert_sn,
                    sandbox,
                )))
            },
        );
        _methods.add_function(
            "set_public_params",
            |lua, (this, params): (LuaAnyUserData, LuaTable)| {
                let this = this.take::<Self>()?;
                let params = table_to_map!(lua, params);
                Ok(AlipayClientWithParams(this.0.set_public_params(params)))
            },
        );
        _methods.add_async_function(
            "post",
            |lua, (this, method, biz_content): (LuaAnyUserData, String, LuaMultiValue)| async move {
                let this = this.take::<Self>()?;
                if !biz_content.is_empty() {
                    let content = biz_content.into_vec();
                    let mut params = HashMap::new();
                    if content.len() == 1 {
                        if let LuaValue::Table(val) = content[0].clone() {
                            params = table_to_map!(lua, val);
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6001,
                                "Parameter error of Alipay post function",
                            ))));
                        }
                    } else {
                        let len = content.len();
                        let mut i = 0;
                        if len % 2 == 0 {
                            while i < len {
                                if let LuaValue::String(key) = content[i].clone() {
                                    let k = key.to_str()?;
                                    let json_value: JsonValue =
                                        lua.from_value(content[i + 1].clone())?;
                                    params.insert(k.to_owned(), json_value);
                                }
                                i += 2;
                            }
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6001,
                                "Parameter error of Alipay post function",
                            ))));
                        }
                    }

                    let data: AlipayResponse = this.0.post(method, params).await.to_lua_err()?;
                    Ok(Response(data))
                } else {
                    let data: AlipayResponse = this.0.no_param_post(method).await.to_lua_err()?;
                    Ok(Response(data))
                }
            },
        );
        _methods.add_function(
            "generate_url_data",
            |lua, (this, method, biz_content): (LuaAnyUserData, String, LuaMultiValue)| {
                let this = this.take::<Self>()?;
                if !biz_content.is_empty() {
                    let content = biz_content.into_vec();
                    let mut params = HashMap::new();
                    if content.len() == 1 {
                        if let LuaValue::Table(val) = content[0].clone() {
                            params = table_to_map!(lua, val);
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6001,
                                "Parameter error of Alipay post function",
                            ))));
                        }
                    } else {
                        let len = content.len();
                        let mut i = 0;
                        if len % 2 == 0 {
                            while i < len {
                                if let LuaValue::String(key) = content[i].clone() {
                                    let k = key.to_str()?;
                                    let json_value: JsonValue =
                                        lua.from_value(content[i + 1].clone())?;
                                    params.insert(k.to_owned(), json_value);
                                }
                                i += 2;
                            }
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6001,
                                "Parameter error of Alipay post function",
                            ))));
                        }
                    }

                    let data = this.0.generate_url_data(method, params).to_lua_err()?;
                    let params = lua.create_table()?;
                    for (key, val) in data {
                        params.set(key, val)?;
                    }
                    Ok(params)
                } else {
                    let data = this.0.generate_url_data(method, ()).to_lua_err()?;
                    let params = lua.create_table()?;
                    for (key, val) in data {
                        params.set(key, val)?;
                    }
                    Ok(params)
                }
            },
        );
    }
}

impl LuaUserData for AlipayClientWithParams {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function(
            "set_public_params",
            |lua, (this, params): (LuaAnyUserData, LuaTable)| {
                let mut this = this.take::<Self>()?;
                let params = table_to_map!(lua, params);
                this.0.set_public_params(params);
                Ok(this)
            },
        );
        _methods.add_async_function(
            "post",
            |lua, (this, method, biz_content): (LuaAnyUserData, String, LuaMultiValue)| async move {
                let mut this = this.take::<Self>()?;
                if !biz_content.is_empty() {
                    let content = biz_content.into_vec();
                    let mut params = HashMap::new();
                    if content.len() == 1 {
                        if let LuaValue::Table(val) = content[0].clone() {
                            params = table_to_map!(lua, val);
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6001,
                                "Parameter error of Alipay post function",
                            ))));
                        }
                    } else {
                        let len = content.len();
                        let mut i = 0;
                        if len % 2 == 0 {
                            while i < len {
                                if let LuaValue::String(key) = content[i].clone() {
                                    let k = key.to_str()?;
                                    let json_value: JsonValue =
                                        lua.from_value(content[i + 1].clone())?;
                                    params.insert(k.to_owned(), json_value);
                                }
                                i += 2;
                            }
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6001,
                                "Parameter error of Alipay post function",
                            ))));
                        }
                    }

                    let data: AlipayResponse = this.0.post(method, params).await.to_lua_err()?;
                    Ok(Response(data))
                } else {
                    let data: AlipayResponse = this.0.no_param_post(method).await.to_lua_err()?;
                    Ok(Response(data))
                }
            },
        );
        _methods.add_function(
            "generate_url_data",
            |lua, (this, method, biz_content): (LuaAnyUserData, String, LuaMultiValue)| {
                let mut this = this.take::<Self>()?;
                if !biz_content.is_empty() {
                    let content = biz_content.into_vec();
                    let mut params = HashMap::new();
                    if content.len() == 1 {
                        if let LuaValue::Table(val) = content[0].clone() {
                            params = table_to_map!(lua, val);
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6001,
                                "Parameter error of Alipay post function",
                            ))));
                        }
                    } else {
                        let len = content.len();
                        let mut i = 0;
                        if len % 2 == 0 {
                            while i < len {
                                if let LuaValue::String(key) = content[i].clone() {
                                    let k = key.to_str()?;
                                    let json_value: JsonValue =
                                        lua.from_value(content[i + 1].clone())?;
                                    params.insert(k.to_owned(), json_value);
                                }
                                i += 2;
                            }
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6001,
                                "Parameter error of Alipay post function",
                            ))));
                        }
                    }

                    let data = this.0.generate_url_data(method, params).to_lua_err()?;
                    let params = lua.create_table()?;
                    for (key, val) in data {
                        params.set(key, val)?;
                    }
                    Ok(params)
                } else {
                    let data = this.0.generate_url_data(method, ()).to_lua_err()?;
                    let params = lua.create_table()?;
                    for (key, val) in data {
                        params.set(key, val)?;
                    }
                    Ok(params)
                }
            },
        );
    }
}

pub struct Response(AlipayResponse);

impl LuaUserData for Response {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("text", |lua, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            lua.create_string(&this.0.into_string().to_lua_err()?)
        });
        _methods.add_function("json", |lua, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let data: JsonValue = this.0.into_json().to_lua_err()?;
            lua.to_value(&data)
        });
    }
}

#[mlua::lua_module]
fn alipay(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    lua.create_proxy::<AlipayClient>()
}
