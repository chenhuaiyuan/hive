use mlua::prelude::*;
use serde_json::Value;
use ureq::Response;

pub struct ReqResponse(pub Response);

impl LuaUserData for ReqResponse {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method("get_url", |lua, this, ()| {
            let data = this.0.get_url();
            lua.create_string(data)
        });

        // todo
        // _methods.add_method("http_version", method)
        _methods.add_method("status", |_, this, ()| Ok(this.0.status()));
        _methods.add_method("status_text", |_, this, ()| {
            Ok(this.0.status_text().to_owned())
        });
        _methods.add_method("header", |_, this, name: String| {
            let data = this.0.header(&name);
            let data = data.map(|v| v.to_owned());
            Ok(data)
        });
        _methods.add_method("headers_names", |_, this, ()| Ok(this.0.headers_names()));
        _methods.add_method("has", |_, this, name: String| Ok(this.0.has(&name)));
        _methods.add_method("all", |_, this, name: String| {
            let data = this.0.all(&name);
            let data = data.iter().map(|&v| v.to_owned()).collect::<Vec<String>>();
            Ok(data)
        });
        _methods.add_method("content_type", |_, this, ()| {
            Ok(this.0.content_type().to_owned())
        });
        _methods.add_method("charset", |_, this, ()| Ok(this.0.charset().to_owned()));
        // todo
        // _methods.add_function("into_reader", function)
        _methods.add_function("into_string", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            this.0.into_string().to_lua_err()
        });
        _methods.add_function("into_json", |lua, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let data = this.0.into_json::<Value>().to_lua_err()?;

            lua.to_value(&data)
        });
    }
}
