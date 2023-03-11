#[cfg(feature = "ws")]
use crate::error::Error as WebError;
#[cfg(feature = "ws")]
use crate::lua::websocket::handle_connection;
use crate::request::{HttpData, Request};
#[cfg(feature = "ws")]
use http::header::{
    CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION, UPGRADE,
};
#[cfg(feature = "ws")]
use http::Method;
#[cfg(feature = "ws")]
use http::{HeaderValue, StatusCode, Version};
use hyper::{Body, Request as HyperRequest};
use mlua::prelude::*;
use serde_json::Value as JsonValue;
use std::net::SocketAddr;
#[cfg(feature = "ws")]
use std::sync::Arc;
#[cfg(feature = "ws")]
use tokio_tungstenite::WebSocketStream;
#[cfg(feature = "ws")]
use tungstenite::handshake::derive_accept_key;
#[cfg(feature = "ws")]
use tungstenite::protocol::Role;

pub struct LuaRequest(Request);

impl LuaRequest {
    pub fn new(req: HyperRequest<Body>, remote_addr: SocketAddr) -> Self {
        Self(Request { req, remote_addr })
    }
}

// 用于处理多维数组
fn generate_table<'lua>(
    lua: &Lua,
    tab: LuaTable<'lua>,
    mut cap: Vec<String>,
    val: JsonValue,
) -> LuaResult<LuaTable<'lua>> {
    if cap.is_empty() {
        return Ok(tab);
    }
    let index: String = cap.remove(0);
    let len: usize = cap.len();
    let num: Result<i32, std::num::ParseIntError> = index.parse::<i32>();
    if let Ok(idx) = num {
        let i: i32 = idx + 1;
        if len == 0 {
            let v: LuaValue = lua.to_value(&val)?;
            tab.set(i, v)?;
            generate_table(lua, tab, cap, JsonValue::Null)
        } else {
            let table: LuaResult<LuaTable> = tab.get(i);
            if let Ok(t) = table {
                let temp: LuaTable = generate_table(lua, t, cap, val)?;
                tab.set(i, temp)?;
                Ok(tab)
            } else {
                let temp_tab: LuaTable = lua.create_table()?;
                let t: LuaTable = generate_table(lua, temp_tab, cap, val)?;
                tab.set(i, t)?;
                Ok(tab)
            }
        }
    } else if len == 0 {
        let v: LuaValue = lua.to_value(&val)?;
        tab.set(index, v)?;
        generate_table(lua, tab, cap, JsonValue::Null)
    } else {
        let table: LuaResult<LuaTable> = tab.get(index.clone());
        if let Ok(t) = table {
            let temp: LuaTable = generate_table(lua, t, cap, val)?;
            tab.set(index, temp)?;
            Ok(tab)
        } else {
            let temp_tab: LuaTable = lua.create_table()?;
            let t: LuaTable = generate_table(lua, temp_tab, cap, val)?;
            tab.set(index, t)?;
            Ok(tab)
        }
    }
}

impl LuaUserData for LuaRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_async_function("params", |lua, this: LuaAnyUserData| async move {
            let this: LuaRequest = this.take::<Self>()?;
            let f1 = |mut param: HttpData<LuaValue<'lua>>,
                      param_key: String,
                      fields: Vec<String>,
                      val| {
                let param_value = param.remove(&param_key);
                if let Some(LuaValue::Table(value)) = param_value {
                    let temp_table = generate_table(lua, value, fields, val)?;
                    param.insert(param_key, LuaValue::Table(temp_table));
                } else {
                    let temp = lua.create_table()?;
                    let temp_table = generate_table(lua, temp, fields, val)?;
                    param.insert(param_key, LuaValue::Table(temp_table));
                }
                Ok(param)
            };
            let f2 = |mut param: HttpData<LuaValue<'lua>>, key, val| {
                let val: LuaValue = lua.to_value(&val)?;
                param.insert(key, val);
                Ok(param)
            };
            let params: std::collections::HashMap<String, LuaValue> =
                this.0.params(f1, f2).await.to_lua_err()?;

            Ok(params)
        });
        _methods.add_method("remote_addr", |_, this, ()| {
            Ok((this.0.remote_addr).to_string())
        });
        _methods.add_method("headers", |lua, this, ()| {
            let headers: LuaTable = lua.create_table()?;
            let headers_raw: &http::HeaderMap = this.0.req.headers();
            for (key, val) in headers_raw {
                let key: String = key.as_str().to_string();
                let val: String = val.to_str().to_lua_err()?.to_string();
                headers.set(key, val)?;
            }
            Ok(headers)
        });
        _methods.add_async_function("form", |lua, this: LuaAnyUserData| async move {
            let this: LuaRequest = this.take::<Self>()?;
            let file_func = |mut param: HttpData<LuaValue<'lua>>, field_name, file| {
                param.insert(field_name, LuaValue::UserData(lua.create_userdata(file)?));
                Ok(param)
            };
            let f1 = |mut param: HttpData<LuaValue<'lua>>,
                      param_key: String,
                      fields: Vec<String>,
                      data: JsonValue| {
                let param_value = param.remove(&param_key);
                if let Some(LuaValue::Table(value)) = param_value {
                    let temp_table = generate_table(lua, value, fields, data)?;
                    param.insert(param_key.to_string(), LuaValue::Table(temp_table));
                } else {
                    let temp = lua.create_table()?;
                    let temp_table = generate_table(lua, temp, fields, data)?;
                    param.insert(param_key.to_string(), LuaValue::Table(temp_table));
                }
                Ok(param)
            };
            let f2 = |mut param: HttpData<LuaValue<'lua>>, field_name, data| {
                let data: LuaValue = lua.to_value(&data)?;
                param.insert(field_name, data);
                Ok(param)
            };
            let params: std::collections::HashMap<String, LuaValue> =
                this.0.form(file_func, f1, f2).await.to_lua_err()?;
            Ok(params)
        });
        #[cfg(feature = "ws")]
        _methods.add_async_function(
            "upgrade",
            |lua, (this, func): (LuaAnyUserData, LuaFunction)| async move {
                let this = this.take::<Self>()?;
                let upgrade = HeaderValue::from_static("Upgrade");
                let websocket = HeaderValue::from_static("websocket");
                let headers = this.0.req.headers();
                let key = headers.get(SEC_WEBSOCKET_KEY);
                let derived = key.map(|k| derive_accept_key(k.as_bytes()));
                if this.0.req.method() != Method::GET
                    || this.0.req.version() < Version::HTTP_11
                    || !headers
                        .get(CONNECTION)
                        .and_then(|h| h.to_str().ok())
                        .map(|h| {
                            h.split(|c| c == ' ' || c == ',')
                                .any(|p| p.eq_ignore_ascii_case(upgrade.to_str().unwrap()))
                        })
                        .unwrap_or(false)
                    || !headers
                        .get(UPGRADE)
                        .and_then(|h| h.to_str().ok())
                        .map(|h| h.eq_ignore_ascii_case("websocket"))
                        .unwrap_or(false)
                    || !headers
                        .get(SEC_WEBSOCKET_VERSION)
                        .map(|h| h == "13")
                        .unwrap_or(false)
                    || key.is_none()
                {
                    Err(LuaError::ExternalError(Arc::new(WebError::new(
                        5047,
                        "Please check whether the parameter transfer is correct",
                    ))))
                } else {
                    // let ver = this.0.version();
                    let mut req = this.0.req;
                    let func: LuaFunction<'static> = unsafe { std::mem::transmute(func) };
                    tokio::task::spawn_local(async move {
                        match hyper::upgrade::on(&mut req).await {
                            Ok(upgraded) => {
                                handle_connection(
                                    func,
                                    WebSocketStream::from_raw_socket(upgraded, Role::Server, None)
                                        .await,
                                    this.0.remote_addr,
                                )
                                .await
                                .unwrap();
                            }
                            Err(e) => println!("upgrade error: {e}"),
                        }
                    });
                    let res = lua.create_table()?;
                    let headers = lua.create_table()?;
                    res.set(
                        "status",
                        LuaValue::Integer(StatusCode::SWITCHING_PROTOCOLS.as_u16() as i64),
                    )?;
                    res.set("version", lua.create_string("HTTP/1.1")?)?;
                    headers.set(CONNECTION.as_str(), lua.create_string(upgrade.as_bytes())?)?;
                    headers.set(UPGRADE.as_str(), lua.create_string(websocket.as_bytes())?)?;
                    headers.set(
                        SEC_WEBSOCKET_ACCEPT.as_str(),
                        lua.create_string(&derived.unwrap())?,
                    )?;
                    res.set("headers", LuaValue::Table(headers))?;
                    Ok(res)
                }
            },
        );
    }
}
