mod error;

use alipay_rs::{Cli, Client, ClientWithParams, MutCli, Response as AlipayResponse};
use error::Error as WebError;
use mlua::prelude::*;
use serde_json::{Map, Value as JsonValue};
use std::sync::Arc;

fn lua_is_array(table: LuaTable) -> LuaResult<bool> {
    let mut is_array = true;
    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, _) = pair?;
        if key.type_name() != "integer" {
            is_array = false;
            break;
        }
    }
    Ok(is_array)
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
        JsonValue::Array(v) => {
            let table = lua.create_table()?;
            let mut i = 1;
            for val in v {
                table.set(i, json_value_to_lua_value(val, lua)?)?;
                i += 1;
            }
            return Ok(LuaValue::Table(table));
        }
        JsonValue::Object(v) => {
            let table = lua.create_table()?;
            for (key, val) in v {
                table.set(key, json_value_to_lua_value(val, lua)?)?;
            }
            return Ok(LuaValue::Table(table));
        }
    }
}

fn lua_value_to_json_value(val: LuaValue, _lua: &Lua) -> LuaResult<JsonValue> {
    match val {
        LuaValue::Nil => Ok(JsonValue::Null),
        LuaValue::Boolean(v) => Ok(JsonValue::Bool(v)),
        LuaValue::Integer(v) => Ok(JsonValue::from(v)),
        LuaValue::Number(v) => Ok(JsonValue::from(v)),
        LuaValue::String(v) => {
            let data = v.to_str()?;
            Ok(JsonValue::from(data))
        }
        LuaValue::Table(v) => {
            let is_array = lua_is_array(v.clone())?;
            if is_array {
                let mut arr: Vec<JsonValue> = Vec::new();
                for pair in v.pairs::<LuaValue, LuaValue>() {
                    let (_, val) = pair?;
                    arr.push(lua_value_to_json_value(val, _lua)?);
                }
                Ok(JsonValue::from(arr))
            } else {
                let mut map: Map<String, JsonValue> = Map::new();
                for pair in v.pairs::<String, LuaValue>() {
                    let (key, val) = pair?;
                    map.insert(key, lua_value_to_json_value(val, _lua)?);
                }
                Ok(JsonValue::from(map))
            }
        }
        _ => Ok(JsonValue::Null),
    }
}

pub struct AlipayClient {
    client: Client,
}

pub struct AlipayClientWithParams {
    client: ClientWithParams,
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

#[mlua::lua_module]
fn alipay(lua: &Lua) -> LuaResult<LuaFunction> {
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
