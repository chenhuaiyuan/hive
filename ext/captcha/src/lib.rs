use captcha_rs::CaptchaBuilder;
use mlua::prelude::*;

pub struct HiveCaptchaBuilder(CaptchaBuilder);

impl LuaUserData for HiveCaptchaBuilder {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("length", |_, (this, length): (LuaAnyUserData, usize)| {
            let this = this.take::<HiveCaptchaBuilder>()?;
            let t = this.0.length(length);
            Ok(HiveCaptchaBuilder(t))
        });
        _methods.add_function("width", |_, (this, width): (LuaAnyUserData, u32)| {
            let this = this.take::<HiveCaptchaBuilder>()?;
            let t = this.0.width(width);
            Ok(HiveCaptchaBuilder(t))
        });
        _methods.add_function("height", |_, (this, height): (LuaAnyUserData, u32)| {
            let this = this.take::<HiveCaptchaBuilder>()?;
            let t = this.0.height(height);
            Ok(HiveCaptchaBuilder(t))
        });
        _methods.add_function(
            "dark_mode",
            |_, (this, dark_mode): (LuaAnyUserData, bool)| {
                let this = this.take::<HiveCaptchaBuilder>()?;
                let t = this.0.dark_mode(dark_mode);
                Ok(HiveCaptchaBuilder(t))
            },
        );
        _methods.add_function(
            "complexity",
            |_, (this, complexity): (LuaAnyUserData, u32)| {
                let this = this.take::<HiveCaptchaBuilder>()?;
                let t = this.0.complexity(complexity);
                Ok(HiveCaptchaBuilder(t))
            },
        );
        _methods.add_function("build", |lua, this: LuaAnyUserData| {
            let this = this.take::<HiveCaptchaBuilder>()?;
            let tab = lua.create_table()?;
            let data = this.0.build();
            tab.set("text", lua.create_string(&data.text)?)?;
            tab.set("base_img", lua.create_string(&data.base_img)?)?;
            Ok(tab)
        });
    }
}

#[mlua::lua_module]
pub fn captcha(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| Ok(HiveCaptchaBuilder(CaptchaBuilder::new())))
}
