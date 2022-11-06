use std::sync::Arc;

use super::{json_value_to_lua_value, lua_value_to_json_value};
use crate::error::Error as WebError;
use alipay_rs::{Cli, Client, ClientWithParams, MutCli, Response as AlipayResponse};
use mlua::prelude::*;
use serde_json::Value as JsonValue;

pub struct AlipayClient {
    client: Client,
}

pub struct AlipayClientWithParams {
    client: ClientWithParams,
}

pub fn create_alipay(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(
        |_,
         (app_id, public_key, private_key, app_cert_sn, alipay_root_cert_sn): (
            String,
            String,
            String,
            String,
            String,
        )| {
            let public_key_data = std::fs::read_to_string(public_key).to_lua_err()?;
            let private_key_data = std::fs::read_to_string(private_key).to_lua_err()?;
            let app_cert_sn_data = std::fs::read_to_string(app_cert_sn).to_lua_err()?;
            let alipay_root_cert_sn_data =
                std::fs::read_to_string(alipay_root_cert_sn).to_lua_err()?;
            let client = Client::builder()
                .app_id(&app_id)
                .public_key(&public_key_data)
                .private_key(&private_key_data)
                .app_cert_sn(&app_cert_sn_data)
                .alipay_root_cert_sn(&alipay_root_cert_sn_data)
                .finish();
            Ok(AlipayClient { client })
        },
    )
}

macro_rules! table_to_vec {
    ($table: expr, $lua: ident) => {{
        let mut data = Vec::new();
        for pair in $table.pairs::<LuaValue, LuaValue>() {
            let (key, val) = pair?;
            data.push((
                lua_value_to_json_value(key, $lua)?,
                lua_value_to_json_value(val, $lua)?,
            ));
        }
        data
    }};
}

impl LuaUserData for AlipayClient {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_async_function(
            "post",
            |lua, (this, method, biz_content): (LuaAnyUserData, String, LuaMultiValue)| async move {
                let this = this.take::<Self>()?;
                if !biz_content.is_empty() {
                    let content = biz_content.into_vec();
                    let mut params = Vec::new();
                    if content.len() == 1 {
                        if let LuaValue::Table(val) = content[0].clone() {
                            params = table_to_vec!(val, lua);
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
                                params.push((
                                    lua_value_to_json_value(content[i].clone(), lua)?,
                                    lua_value_to_json_value(content[i + 1].clone(), lua)?,
                                ));
                                i += 2;
                            }
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6001,
                                "Parameter error of Alipay post function",
                            ))));
                        }
                    }

                    let data: AlipayResponse =
                        this.client.post(method, params).await.to_lua_err()?;
                    Ok(Response(data))
                } else {
                    let data: AlipayResponse =
                        this.client.no_param_post(method).await.to_lua_err()?;
                    Ok(Response(data))
                }
            },
        );
        _methods.add_function(
            "set_public_params",
            |lua, (this, params): (LuaAnyUserData, LuaTable)| {
                let this = this.take::<Self>()?;
                let params = table_to_vec!(params, lua);
                Ok(AlipayClientWithParams {
                    client: this.client.set_public_params(params),
                })
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
                let params = table_to_vec!(params, lua);
                this.client.set_public_params(params);
                Ok(this)
            },
        );
        _methods.add_async_function(
            "post",
            |lua, (this, method, biz_content): (LuaAnyUserData, String, LuaMultiValue)| async move {
                let mut this = this.take::<Self>()?;
                if !biz_content.is_empty() {
                    let content = biz_content.into_vec();
                    let mut params = Vec::new();
                    if content.len() == 1 {
                        if let LuaValue::Table(val) = content[0].clone() {
                            params = table_to_vec!(val, lua);
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
                                params.push((
                                    lua_value_to_json_value(content[i].clone(), lua)?,
                                    lua_value_to_json_value(content[i + 1].clone(), lua)?,
                                ));
                                i += 2;
                            }
                        } else {
                            return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                6001,
                                "Parameter error of Alipay post function",
                            ))));
                        }
                    }

                    let data: AlipayResponse =
                        this.client.post(method, params).await.to_lua_err()?;
                    Ok(Response(data))
                } else {
                    let data: AlipayResponse =
                        this.client.no_param_post(method).await.to_lua_err()?;
                    Ok(Response(data))
                }
            },
        )
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
            json_value_to_lua_value(data, lua)
        });
    }
}
