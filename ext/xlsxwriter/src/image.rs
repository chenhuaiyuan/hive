use mlua::prelude::*;
use rust_xlsxwriter::Image;

pub struct XlsxImage(pub Image);

pub fn create_image(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, path: String| {
        let image = Image::new(path).to_lua_err()?;
        Ok(XlsxImage(image))
    })
}

impl LuaUserData for XlsxImage {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method_mut("set_scale_height", |_, this, scale: f64| {
            this.0.set_scale_height(scale);
            Ok(())
        });
        _methods.add_method_mut("set_scale_width", |_, this, scale: f64| {
            this.0.set_scale_width(scale);
            Ok(())
        });
        _methods.add_method_mut("set_alt_text", |_, this, alt_text: String| {
            this.0.set_alt_text(&alt_text);
            Ok(())
        });
        _methods.add_method_mut("set_decorative", |_, this, enable: bool| {
            this.0.set_decorative(enable);
            Ok(())
        });
        _methods.add_method("width", |_, this, ()| Ok(this.0.width()));
        _methods.add_method("height", |_, this, ()| Ok(this.0.height()));
        _methods.add_method("width_dpi", |_, this, ()| Ok(this.0.width_dpi()));
        _methods.add_method("height_dpi", |_, this, ()| Ok(this.0.height_dpi()));
    }
}
