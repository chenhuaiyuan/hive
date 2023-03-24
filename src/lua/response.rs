use http::{response::Builder, Response, Version};
use hyper::Body;
use mlua::prelude::*;

pub struct HiveResponseBuilder(Builder);

impl LuaUserData for HiveResponseBuilder {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("new", |_, ()| Ok(HiveResponseBuilder(Response::builder())));
        _methods.add_function("body", |_, (this, body): (LuaAnyUserData, LuaValue)| {
            let this = this.take::<Self>()?;
            let resp = match body {
                LuaValue::String(v) => {
                    let data = v.as_bytes().to_vec();
                    this.0.body(Body::from(data)).to_lua_err()?
                }
                _ => {
                    let body = serde_json::to_vec(&body).to_lua_err()?;
                    this.0.body(Body::from(body)).to_lua_err()?
                }
            };
            Ok(HiveResponse(resp))
        });
        _methods.add_function("status", |_, (this, status): (LuaAnyUserData, u16)| {
            let this = this.take::<Self>()?;
            Ok(HiveResponseBuilder(this.0.status(status)))
        });
        _methods.add_function("version", |_, (this, version): (LuaAnyUserData, String)| {
            let this = this.take::<Self>()?;
            let ver = if version == "HTTP/0.9" {
                Version::HTTP_09
            } else if version == "HTTP/1.0" {
                Version::HTTP_10
            } else if version == "HTTP/2.0" {
                Version::HTTP_2
            } else if version == "HTTP/3.0" {
                Version::HTTP_3
            } else {
                Version::HTTP_11
            };
            Ok(HiveResponseBuilder(this.0.version(ver)))
        });
        _methods.add_function(
            "headers",
            |_, (this, headers): (LuaAnyUserData, LuaTable)| {
                let this = this.take::<Self>()?;
                let mut resp = this.0;
                for pair in headers.pairs::<String, String>() {
                    let (h, v) = pair.to_lua_err()?;
                    resp = resp.header(h, v);
                }
                Ok(HiveResponseBuilder(resp))
            },
        );
    }
}

pub struct HiveResponse<T>(pub Response<T>);

impl<T> LuaUserData for HiveResponse<T> {}
