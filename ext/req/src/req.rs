use std::{fs::File, io::BufReader};

use cookie_store::CookieStore;
use ureq::{Agent, AgentBuilder, Proxy, RedirectAuthHeaders};

use hive_time::TimeDuration;
use mlua::prelude::*;

use crate::request::ReqRequest;

pub struct Req(AgentBuilder);

pub fn create_req(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| Ok(Req(AgentBuilder::new())))
}

impl LuaUserData for Req {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function(
            "proxy",
            |_, (this, proxy): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let proxy = proxy.take::<ReqProxy>()?;
                Ok(Req(this.0.proxy(proxy.0)))
            },
        );
        _methods.add_function(
            "https_only",
            |_, (this, enforce): (LuaAnyUserData, bool)| {
                let this = this.take::<Self>()?;
                Ok(Req(this.0.https_only(enforce)))
            },
        );
        _methods.add_function(
            "max_idle_connections",
            |_, (this, max): (LuaAnyUserData, usize)| {
                let this = this.take::<Self>()?;
                Ok(Req(this.0.max_idle_connections(max)))
            },
        );
        _methods.add_function(
            "max_idle_connections_per_host",
            |_, (this, max): (LuaAnyUserData, usize)| {
                let this = this.take::<Self>()?;
                Ok(Req(this.0.max_idle_connections_per_host(max)))
            },
        );
        // todo
        // _methods.add_function("resolver", |_, (this, resolver): (LuaAnyUserData, )|)
        _methods.add_function(
            "timeout_connect",
            |_, (this, timeout): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let timeout = timeout.take::<TimeDuration>()?;
                Ok(Req(this.0.timeout_connect(timeout.0)))
            },
        );
        _methods.add_function(
            "timeout_read",
            |_, (this, timeout): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let timeout = timeout.take::<TimeDuration>()?;
                Ok(Req(this.0.timeout_read(timeout.0)))
            },
        );
        _methods.add_function(
            "timeout_write",
            |_, (this, timeout): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let timeout = timeout.take::<TimeDuration>()?;
                Ok(Req(this.0.timeout_write(timeout.0)))
            },
        );
        _methods.add_function(
            "timeout",
            |_, (this, timeout): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let timeout = timeout.take::<TimeDuration>()?;
                Ok(Req(this.0.timeout(timeout.0)))
            },
        );
        _methods.add_function("no_delay", |_, (this, no_delay): (LuaAnyUserData, bool)| {
            let this = this.take::<Self>()?;
            Ok(Req(this.0.no_delay(no_delay)))
        });
        _methods.add_function("redirects", |_, (this, n): (LuaAnyUserData, u32)| {
            let this = this.take::<Self>()?;
            Ok(Req(this.0.redirects(n)))
        });
        _methods.add_function(
            "redirect_auth_headers",
            |_, (this, v): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let v = v.take::<ReqRedirectAuthHeaders>()?;
                Ok(Req(this.0.redirect_auth_headers(v.0)))
            },
        );
        _methods.add_function(
            "user_agent",
            |_, (this, user_agent): (LuaAnyUserData, String)| {
                let this = this.take::<Self>()?;
                Ok(Req(this.0.user_agent(&user_agent)))
            },
        );
        // todo
        // _methods.add_function("tls_config", function)
        // _methods.add_function("tls_connector", function)
        // _methods.add_function(
        //     "cookie_store",
        //     |_, (this, file_name): (LuaAnyUserData, String)| {
        //         let this = this.take::<Self>()?;
        //         let file = File::open(file_name).to_lua_err()?;
        //         let read = BufReader::new(file);
        //         let store = CookieStore::load_json(read).to_lua_err()?;
        //         Ok(Req(this.0.cookie_store(store)))
        //     },
        // );
        _methods.add_function("build", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let agent = this.0.build();
            Ok(ReqAgent(agent))
        });
    }
}

pub struct ReqProxy(Proxy);

pub fn create_proxy(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, proxy: String| Ok(ReqProxy(Proxy::new(proxy).to_lua_err()?)))
}

impl LuaUserData for ReqProxy {}

pub struct ReqRedirectAuthHeaders(RedirectAuthHeaders);

pub fn create_redirect_auth_headers(lua: &Lua) -> LuaResult<LuaTable> {
    lua.create_table_from([
        (
            "never",
            lua.create_userdata(ReqRedirectAuthHeaders(RedirectAuthHeaders::Never))?,
        ),
        (
            "same_host",
            lua.create_userdata(ReqRedirectAuthHeaders(RedirectAuthHeaders::SameHost))?,
        ),
    ])
}

impl LuaUserData for ReqRedirectAuthHeaders {}

pub struct ReqAgent(Agent);

impl LuaUserData for ReqAgent {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method("request", |_, this, (method, path): (String, String)| {
            let req = this.0.request(&method, &path);
            Ok(ReqRequest(req))
        });
        _methods.add_method("get", |_, this, path: String| {
            Ok(ReqRequest(this.0.get(&path)))
        });
        _methods.add_method("head", |_, this, path: String| {
            Ok(ReqRequest(this.0.head(&path)))
        });
        _methods.add_method("patch", |_, this, path: String| {
            Ok(ReqRequest(this.0.patch(&path)))
        });
        _methods.add_method("post", |_, this, path: String| {
            Ok(ReqRequest(this.0.post(&path)))
        });
        _methods.add_method("put", |_, this, path: String| {
            Ok(ReqRequest(this.0.put(&path)))
        });
        _methods.add_method("delete", |_, this, path: String| {
            Ok(ReqRequest(this.0.delete(&path)))
        });
        _methods.add_method("cookie_store", |_, this, file_name: String| {
            let mut file = File::create(file_name).to_lua_err()?;
            this.0.cookie_store().save_json(&mut file).to_lua_err()?;
            Ok(())
        });
    }
}

pub fn create_delete(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, path: String| {
        let req = ureq::delete(&path);
        Ok(ReqRequest(req))
    })
}

pub fn create_get(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, path: String| {
        let req = ureq::get(&path);
        Ok(ReqRequest(req))
    })
}

pub fn create_head(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, path: String| {
        let req = ureq::head(&path);
        Ok(ReqRequest(req))
    })
}

pub fn create_patch(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, path: String| {
        let req = ureq::patch(&path);
        Ok(ReqRequest(req))
    })
}

pub fn create_post(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, path: String| {
        let req = ureq::post(&path);
        Ok(ReqRequest(req))
    })
}

pub fn create_put(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, path: String| {
        let req = ureq::put(&path);
        Ok(ReqRequest(req))
    })
}
