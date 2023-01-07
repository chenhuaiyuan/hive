use super::file_data::FileData;
use crate::error::Error as WebError;
#[cfg(feature = "ws")]
use crate::lua::websocket::handle_connection;
#[cfg(feature = "ws")]
use http::header::{
    CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION, UPGRADE,
};
use http::{header, header::CONTENT_TYPE, HeaderMap, Method};
#[cfg(feature = "ws")]
use http::{HeaderValue, StatusCode, Version};
use hyper::{Body, Request};
use mlua::prelude::*;
use multer::Multipart;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(feature = "ws")]
use tokio_tungstenite::WebSocketStream;
#[cfg(feature = "ws")]
use tungstenite::handshake::derive_accept_key;
#[cfg(feature = "ws")]
use tungstenite::protocol::Role;

fn has_content_type(headers: &HeaderMap, expected_content_type: &mime::Mime) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    content_type.starts_with(expected_content_type.as_ref())
}

pub struct LuaRequest(Request<Body>, SocketAddr);

impl LuaRequest {
    pub fn new(req: Request<Body>, addr: SocketAddr) -> Self {
        LuaRequest(req, addr)
    }
}

fn generate_table<'lua>(
    lua: &Lua,
    tab: LuaTable<'lua>,
    mut cap: Vec<&str>,
    val: String,
) -> LuaResult<LuaTable<'lua>> {
    if cap.is_empty() {
        return Ok(tab);
    }
    let index = cap.remove(0);
    let len = cap.len();
    let num = index.parse::<i32>();
    if let Ok(idx) = num {
        let i = idx + 1;
        if len == 0 {
            tab.set(i, val)?;
            generate_table(lua, tab, cap, "".to_owned())
        } else {
            let table: LuaResult<LuaTable> = tab.get(i);
            if let Ok(t) = table {
                let temp = generate_table(lua, t, cap, val)?;
                tab.set(i, temp)?;
                Ok(tab)
            } else {
                let temp_tab = lua.create_table()?;
                let t = generate_table(lua, temp_tab, cap, val)?;
                tab.set(i, t)?;
                Ok(tab)
            }
        }
    } else if len == 0 {
        tab.set(index, val)?;
        generate_table(lua, tab, cap, "".to_owned())
    } else {
        let table: LuaResult<LuaTable> = tab.get(index);
        if let Ok(t) = table {
            let temp = generate_table(lua, t, cap, val)?;
            tab.set(index, temp)?;
            Ok(tab)
        } else {
            let temp_tab = lua.create_table()?;
            let t = generate_table(lua, temp_tab, cap, val)?;
            tab.set(index, t)?;
            Ok(tab)
        }
    }
}

impl LuaUserData for LuaRequest {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_async_function("params", |lua, this: LuaAnyUserData| async move {
            let params_table = lua.create_table()?;
            let this = this.take::<Self>()?;
            if this.0.method() == Method::GET {
                let query = this.0.uri().query().unwrap_or_default();
                let value = serde_urlencoded::from_str::<Vec<(String, String)>>(query)
                    .map_err(WebError::parse_params)
                    .to_lua_err()?;

                let mut param: HashMap<String, LuaValue> = HashMap::new();
                for (key, val) in value {
                    let offset = key.find('[');
                    if let Some(o) = offset {
                        let k = key.get(0..o);
                        if let Some(k) = k {
                            let roffset = key.rfind(']');
                            if let Some(r) = roffset {
                                let field = key.get((o + 1)..r);
                                if let Some(field) = field {
                                    let fields: Vec<&str> = field.split("][").collect();
                                    let tab = param.get(k);
                                    if let Some(LuaValue::Table(t)) = tab {
                                        let temp_table =
                                            generate_table(lua, t.clone(), fields, val)?;
                                        param.insert(k.to_owned(), LuaValue::Table(temp_table));
                                    } else {
                                        let temp = lua.create_table()?;
                                        let temp_table = generate_table(lua, temp, fields, val)?;
                                        param.insert(k.to_owned(), LuaValue::Table(temp_table));
                                    }
                                } else {
                                    return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                        5031,
                                        "The transmitted parameters are incorrect",
                                    ))));
                                }
                            } else {
                                return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                    5031,
                                    "The transmitted parameters are incorrect",
                                ))));
                            }
                        }
                    } else {
                        param.insert(key, LuaValue::String(lua.create_string(&val)?));
                    }
                }
                for (key, val) in param {
                    params_table.set(key, val)?;
                }
                log::info!(
                    "params: {}",
                    serde_json::to_string(&params_table).to_lua_err()?
                );
                Ok(params_table)
            } else {
                if !has_content_type(this.0.headers(), &mime::APPLICATION_WWW_FORM_URLENCODED) {
                    return Ok(params_table);
                }
                let bytes = hyper::body::to_bytes(this.0).await.to_lua_err()?;
                let value = serde_urlencoded::from_bytes::<Vec<(String, String)>>(&bytes)
                    .map_err(WebError::parse_params)
                    .to_lua_err()?;

                let mut param: HashMap<String, LuaValue> = HashMap::new();
                for (key, val) in value {
                    let offset = key.find('[');
                    if let Some(o) = offset {
                        let k = key.get(0..o);
                        if let Some(k) = k {
                            let roffset = key.rfind(']');
                            if let Some(r) = roffset {
                                let field = key.get((o + 1)..r);
                                if let Some(field) = field {
                                    let fields: Vec<&str> = field.split("][").collect();
                                    let tab = param.get(k);
                                    if let Some(LuaValue::Table(t)) = tab {
                                        let temp_table =
                                            generate_table(lua, t.clone(), fields, val)?;
                                        param.insert(k.to_owned(), LuaValue::Table(temp_table));
                                    } else {
                                        let temp = lua.create_table()?;
                                        let temp_table = generate_table(lua, temp, fields, val)?;
                                        param.insert(k.to_owned(), LuaValue::Table(temp_table));
                                    }
                                } else {
                                    return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                        5031,
                                        "The transmitted parameters are incorrect",
                                    ))));
                                }
                            } else {
                                return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                    5031,
                                    "The transmitted parameters are incorrect",
                                ))));
                            }
                        }
                    } else {
                        param.insert(key, LuaValue::String(lua.create_string(&val)?));
                    }
                }
                for (key, val) in param {
                    params_table.set(key, val)?;
                }
                log::info!(
                    "params: {}",
                    serde_json::to_string(&params_table).to_lua_err()?
                );
                Ok(params_table)
            }
        });
        _methods.add_method("remote_addr", |_, this, ()| Ok((this.1).to_string()));
        _methods.add_method("headers", |lua, this, ()| {
            let headers = lua.create_table()?;
            let headers_raw = this.0.headers();
            for (key, val) in headers_raw {
                let key = key.as_str().to_string();
                let val = val.to_str().to_lua_err()?.to_string();
                headers.set(key, val)?;
            }
            Ok(headers)
        });
        _methods.add_async_function("form", |lua, this: LuaAnyUserData| async move {
            let form_data = lua.create_table()?;
            let this = this.take::<Self>()?;
            if !has_content_type(this.0.headers(), &mime::MULTIPART_FORM_DATA) {
                return Ok(form_data);
            }
            let boundary = this
                .0
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|ct| ct.to_str().ok())
                .and_then(|ct| multer::parse_boundary(ct).ok());

            if boundary.is_none() {
                return Err(LuaError::ExternalError(Arc::new(WebError::new(
                    5041,
                    "no multipart boundary was found",
                ))));
            }

            let mut multipart = Multipart::new(this.0.into_body(), boundary.unwrap());

            let mut param: HashMap<String, LuaValue> = HashMap::new();

            while let Some(mut field) = multipart.next_field().await.to_lua_err()? {
                let name = field.name().map(|v| v.to_string());

                let file_name = field.file_name().map(|v| v.to_string());

                let content_type = field.content_type().map(|v| v.to_string());

                // println!(
                //     "Name: {:?}, FileName: {:?}, Content-Type: {:?}",
                //     name, file_name, content_type
                // );

                let mut field_data: Vec<u8> = Vec::new();
                // let mut field_bytes_len = 0;
                while let Some(field_chunk) = field.chunk().await.to_lua_err()? {
                    // Do something with field chunk.
                    // field_bytes_len += field_chunk.len();
                    field_data.append(&mut field_chunk.to_vec());
                    // println!("{:?}", field_chunk);
                }

                if let Some(file_name) = file_name.clone() {
                    let field_name = name.clone().unwrap_or_else(|| "default".to_string());
                    let content_type = content_type
                        .clone()
                        .unwrap_or_else(|| "image/jpeg".to_string());
                    let file =
                        FileData::new(field_name.clone(), file_name, content_type, field_data);
                    form_data.set(field_name, lua.create_userdata(file)?)?;
                } else if let Some(field_name) = name.clone() {
                    let data = String::from_utf8(field_data).to_lua_err()?;
                    let offset = field_name.find('[');
                    if let Some(o) = offset {
                        let k = field_name.get(0..o);
                        if let Some(k) = k {
                            let roffset = field_name.rfind(']');
                            if let Some(r) = roffset {
                                let field = field_name.get((o + 1)..r);
                                if let Some(field) = field {
                                    let fields: Vec<&str> = field.split("][").collect();
                                    let tab = param.get(k);
                                    if let Some(LuaValue::Table(t)) = tab {
                                        let temp_table =
                                            generate_table(lua, t.clone(), fields, data)?;
                                        param.insert(k.to_owned(), LuaValue::Table(temp_table));
                                    } else {
                                        let temp = lua.create_table()?;
                                        let temp_table = generate_table(lua, temp, fields, data)?;
                                        param.insert(k.to_owned(), LuaValue::Table(temp_table));
                                    }
                                } else {
                                    return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                        5031,
                                        "The transmitted parameters are incorrect",
                                    ))));
                                }
                            } else {
                                return Err(LuaError::ExternalError(Arc::new(WebError::new(
                                    5031,
                                    "The transmitted parameters are incorrect",
                                ))));
                            }
                        }
                    } else {
                        param.insert(field_name, LuaValue::String(lua.create_string(&data)?));
                    }
                }
            }

            for (key, val) in param {
                form_data.set(key, val)?;
            }
            // file没有实现序列化，所以不能使用，后期再去实现，TODO
            // log::info!(
            //     "form data: {}",
            //     serde_json::to_string(&form_data).to_lua_err()?
            // );
            Ok(form_data)
        });
        #[cfg(feature = "ws")]
        _methods.add_async_function(
            "upgrade",
            |lua, (this, func): (LuaAnyUserData, LuaFunction)| async move {
                let this = this.take::<Self>()?;
                let upgrade = HeaderValue::from_static("Upgrade");
                let websocket = HeaderValue::from_static("websocket");
                let headers = this.0.headers();
                let key = headers.get(SEC_WEBSOCKET_KEY);
                let derived = key.map(|k| derive_accept_key(k.as_bytes()));
                if this.0.method() != Method::GET
                    || this.0.version() < Version::HTTP_11
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
                    let mut req = this.0;
                    let func: LuaFunction<'static> = unsafe { std::mem::transmute(func) };
                    tokio::task::spawn_local(async move {
                        match hyper::upgrade::on(&mut req).await {
                            Ok(upgraded) => {
                                handle_connection(
                                    func,
                                    WebSocketStream::from_raw_socket(upgraded, Role::Server, None)
                                        .await,
                                    this.1,
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
