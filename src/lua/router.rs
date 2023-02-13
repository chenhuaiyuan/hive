use mlua::prelude::*;
use std::collections::HashMap;

// Router<(function, middleware)>
type Router =
    HashMap<String, matchit::Router<(LuaFunction<'static>, Option<LuaFunction<'static>>)>>;

pub struct HiveRouter(Router);

impl LuaUserData for HiveRouter {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("new", |_, ()| {
            let router = Router::new();
            Ok(HiveRouter(router))
        });
        _methods.add_method_mut(
            "match",
            |_,
             this,
             (method, path, func, middleware): (
                String,
                String,
                LuaFunction,
                Option<LuaFunction>,
            )| {
                let func: LuaFunction<'static> = unsafe { std::mem::transmute(func) };
                if let Some(middleware) = middleware {
                    let middleware: LuaFunction<'static> =
                        unsafe { std::mem::transmute(middleware) };
                    this.0
                        .entry(method.to_uppercase())
                        .or_default()
                        .insert(path, (func, Some(middleware)))
                        .to_lua_err()?;
                } else {
                    this.0
                        .entry(method.to_uppercase())
                        .or_default()
                        .insert(path, (func, None))
                        .to_lua_err()?;
                }
                Ok(())
            },
        );
        _methods.add_method("execute", |lua, this, (method, path): (String, String)| {
            let value = this.0.get(&method.to_uppercase());
            let table = lua.create_table()?;
            if let Some(router) = value {
                let matched = router.at(&path);
                if let Ok(matched) = matched {
                    let (func, middleware) = matched.value;
                    let params = lua.create_table()?;
                    let router_params = matched.params;
                    for (key, val) in router_params.iter() {
                        params.set(key, val)?;
                    }
                    table.set("is_exist", true)?;
                    table.set("func", func.clone())?;
                    table.set("middleware", middleware.clone())?;
                    table.set("router_params", params)?;
                    Ok(table)
                } else {
                    table.set("is_exist", false)?;
                    table.set("func", LuaValue::Nil)?;
                    table.set("middleware", LuaValue::Nil)?;
                    table.set("router_params", LuaValue::Nil)?;
                    Ok(table)
                }
            } else {
                table.set("is_exist", false)?;
                table.set("func", LuaValue::Nil)?;
                table.set("middleware", LuaValue::Nil)?;
                table.set("router_params", LuaValue::Nil)?;
                Ok(table)
            }
        });
    }
}

pub fn create_router(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    lua.create_proxy::<HiveRouter>()
}
