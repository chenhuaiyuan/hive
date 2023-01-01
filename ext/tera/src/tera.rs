use std::path::Path;

use hive_base::lua_value_to_json_value;
use mlua::prelude::*;
use tera::{Context, Tera};

pub struct TeraSelf(Tera);

impl LuaUserData for TeraSelf {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("new", |_, dir: String| {
            Ok(TeraSelf(Tera::new(&dir).to_lua_err()?))
        });
        _methods.add_function("parse", |_, dir: String| {
            Ok(TeraSelf(Tera::parse(&dir).to_lua_err()?))
        });
        _methods.add_method(
            "render",
            |_, this, (template_name, context): (String, LuaTable)| {
                let context = lua_value_to_json_value(LuaValue::Table(context))?;
                let context = Context::from_value(context).to_lua_err()?;
                let html = this.0.render(&template_name, &context).to_lua_err()?;
                Ok(html)
            },
        );
        _methods.add_function(
            "one_off",
            |_, (input, context, autoescape): (String, LuaTable, bool)| {
                let context = lua_value_to_json_value(LuaValue::Table(context))?;
                let context = Context::from_value(context).to_lua_err()?;
                let html = Tera::one_off(&input, &context, autoescape).to_lua_err()?;
                Ok(html)
            },
        );
        _methods.add_method("get_template_names", |lua, this, ()| {
            let names = this.0.get_template_names();
            let tab = lua.create_table()?;
            for name in names {
                tab.push(name.to_string())?;
            }
            Ok(tab)
        });
        _methods.add_method_mut(
            "add_raw_template",
            |_, this, (name, content): (String, String)| {
                this.0.add_raw_template(&name, &content).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut("add_raw_templates", |_, this, data: LuaTable| {
            let mut templates: Vec<(String, String)> = Vec::new();
            for pairs in data.pairs::<String, String>() {
                let (key, val) = pairs?;
                templates.push((key, val));
            }
            this.0.add_raw_templates(templates).to_lua_err()?;
            Ok(())
        });
        _methods.add_method_mut(
            "add_template_file",
            |_, this, (path, name): (String, Option<String>)| {
                let path = Path::new(&path);
                if let Some(name) = name {
                    this.0.add_template_file(path, Some(&name)).to_lua_err()?;
                } else {
                    this.0.add_template_file(path, None).to_lua_err()?;
                }
                Ok(())
            },
        );
        // todo
        // _methods.add_method_mut("autoescape_on", |_, this, suffixes: Vec<String>| {
        //     let mut suffs: Vec<&'static str> = Vec::new();
        //     for suff in suffixes {
        //         suffs.push(suff.as_str());
        //     }
        //     this.0.autoescape_on(suffs);
        //     Ok(())
        // });
        // todo
        _methods.add_method_mut("full_reload", |_, this, ()| {
            this.0.full_reload().to_lua_err()?;
            Ok(())
        });
        _methods.add_method_mut("extend", |_, this, other: LuaAnyUserData| {
            let other = other.borrow::<TeraSelf>()?;
            this.0.extend(&other.0).to_lua_err()?;
            Ok(())
        });
    }
}

pub fn create_tera(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    lua.create_proxy::<TeraSelf>()
}

pub fn create_escape_html(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, input: String| Ok(tera::escape_html(&input)))
}
