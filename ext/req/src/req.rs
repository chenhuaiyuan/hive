use ureq::{AgentBuilder, Proxy, RedirectAuthHeaders};

use hive_time::TimeDuration;
use mlua::prelude::*;

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
            "timeout",
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
