use crate::error::Result;
use mlua::prelude::*;
use std::collections::HashMap;

// Router<(function, middleware)>
type Router =
    HashMap<String, matchit::Router<(LuaFunction<'static>, Option<LuaFunction<'static>>)>>;

pub struct HiveRouter(Router);

impl HiveRouter {
    #[allow(dead_code)]
    pub async fn execute<'a>(
        &'a self,
        method: String,
        path: String,
        request: LuaAnyUserData<'a>,
        _exception: LuaFunction<'a>,
        _next: Option<LuaFunction<'a>>,
    ) -> Result<LuaValue> {
        let value = self.0.get(&method.to_uppercase());
        if let Some(router) = value {
            let matched = router.at(&path);
            if let Ok(matched) = matched {
                let (func, middleware) = matched.value;
                let router_params = matched.params;
                let mut params: HashMap<&str, &str> = HashMap::new();
                for (key, val) in router_params.iter() {
                    params.insert(key, val);
                }

                #[cfg(not(feature = "add_entrance"))]
                {
                    let mut req =
                        HashMap::from([("_request", LuaValue::UserData(request.clone()))]);
                    if let Some(middleware) = middleware {
                        let (is_pass, user): (bool, LuaValue) =
                            middleware.call_async(request).await?;
                        if is_pass {
                            req.insert("_user_info", user);
                        } else {
                            let data = _exception
                                .call_async((5001, "Failed to verify token"))
                                .await?;
                            return Ok(data);
                        }
                    }
                    let data = func.call_async((req, params)).await?;
                    Ok(data)
                }
                #[cfg(feature = "add_entrance")]
                {
                    if let Some(_next) = _next {
                        let data = _next
                            .call_async((true, func.clone(), middleware.clone(), request, params))
                            .await?;
                        Ok(data)
                    } else {
                        Ok(LuaValue::Nil)
                    }
                }
            } else {
                #[cfg(not(feature = "add_entrance"))]
                {
                    let data = _exception.call_async((404, "Not Found", 404)).await?;
                    Ok(data)
                }
                #[cfg(feature = "add_entrance")]
                {
                    if let Some(_next) = _next {
                        let data = _next
                            .call_async((
                                false,
                                LuaValue::Nil,
                                LuaValue::Nil,
                                LuaValue::Nil,
                                LuaValue::Nil,
                            ))
                            .await?;
                        Ok(data)
                    } else {
                        Ok(LuaValue::Nil)
                    }
                }
            }
        } else {
            #[cfg(not(feature = "add_entrance"))]
            {
                let data = _exception.call_async((404, "Not Found", 404)).await?;
                Ok(data)
            }
            #[cfg(feature = "add_entrance")]
            {
                if let Some(_next) = _next {
                    let data = _next
                        .call_async((
                            false,
                            LuaValue::Nil,
                            LuaValue::Nil,
                            LuaValue::Nil,
                            LuaValue::Nil,
                        ))
                        .await?;
                    Ok(data)
                } else {
                    Ok(LuaValue::Nil)
                }
            }
        }
    }
}

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
