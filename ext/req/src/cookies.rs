use cookie_store::CookieStore;
use mlua::prelude::*;
use url::Url;

pub struct ReqCookieStore(CookieStore);

impl LuaUserData for ReqCookieStore {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method("get_request_values", |lua, this, url: String| {
            let url = Url::parse(&url).to_lua_err()?;
            let data = this.0.get_request_values(&url);
            let tabs = lua.create_table()?;
            for v in data {
                let tab = lua.create_table()?;
                tab.set(v.0, v.1);
                tabs.push(tab)
            }
            Ok(tabs)
        });
    }
}
