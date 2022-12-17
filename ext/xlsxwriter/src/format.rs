use mlua::prelude::*;
use rust_xlsxwriter::Format;

use crate::{
    XlsxAlign, XlsxBorder, XlsxColor, XlsxDiagonalBorder, XlsxPattern, XlsxScript, XlsxUnderline,
};

pub struct XlsxFormat(pub Format);

pub fn create_format(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| Ok(XlsxFormat(Format::new())))
}

impl LuaUserData for XlsxFormat {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function(
            "set_num_format",
            |_, (this, num_format): (LuaAnyUserData, String)| {
                let this = this.take::<Self>()?;
                let format = this.0.set_num_format(&num_format);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_num_format_index",
            |_, (this, num_format_index): (LuaAnyUserData, u8)| {
                let this = this.take::<Self>()?;
                let format = this.0.set_num_format_index(num_format_index);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function("set_bold", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.set_bold();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("set_italic", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.set_italic();
            Ok(XlsxFormat(format))
        });
        _methods.add_function(
            "set_font_color",
            |_, (this, font_color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = font_color.borrow::<XlsxColor>()?;
                let format = this.0.set_font_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_font_name",
            |_, (this, font_name): (LuaAnyUserData, String)| {
                let this = this.take::<Self>()?;
                let format = this.0.set_font_name(&font_name);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_font_size",
            |_, (this, font_size): (LuaAnyUserData, f64)| {
                let this = this.take::<Self>()?;
                let format = this.0.set_font_size(font_size);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_font_scheme",
            |_, (this, font_scheme): (LuaAnyUserData, String)| {
                let this = this.take::<Self>()?;
                let format = this.0.set_font_scheme(&font_scheme);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_font_family",
            |_, (this, font_family): (LuaAnyUserData, u8)| {
                let this = this.take::<Self>()?;
                let format = this.0.set_font_family(font_family);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_font_charset",
            |_, (this, font_charset): (LuaAnyUserData, u8)| {
                let this = this.take::<Self>()?;
                let format = this.0.set_font_charset(font_charset);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_underline",
            |_, (this, underline): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let underline = underline.borrow::<XlsxUnderline>()?;
                let format = this.0.set_underline(underline.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function("set_font_strikethrough", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.set_font_strikethrough();
            Ok(XlsxFormat(format))
        });
        _methods.add_function(
            "set_font_script",
            |_, (this, font_script): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let script = font_script.borrow::<XlsxScript>()?;
                let format = this.0.set_font_script(script.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_align",
            |_, (this, align): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let align = align.borrow::<XlsxAlign>()?;
                let format = this.0.set_align(align.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function("set_text_wrap", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.set_text_wrap();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("set_indent", |_, (this, indent): (LuaAnyUserData, u8)| {
            let this = this.take::<Self>()?;
            let format = this.0.set_indent(indent);
            Ok(XlsxFormat(format))
        });
        _methods.add_function(
            "set_rotation",
            |_, (this, rotation): (LuaAnyUserData, i16)| {
                let this = this.take::<Self>()?;
                let format = this.0.set_rotation(rotation);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_reading_direction",
            |_, (this, reading_direction): (LuaAnyUserData, u8)| {
                let this = this.take::<Self>()?;
                let format = this.0.set_reading_direction(reading_direction);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function("set_shrink", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.set_shrink();
            Ok(XlsxFormat(format))
        });
        _methods.add_function(
            "set_pattern",
            |_, (this, pattern): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let pattern = pattern.borrow::<XlsxPattern>()?;
                let format = this.0.set_pattern(pattern.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_background_color",
            |_, (this, background_color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let background_color = background_color.borrow::<XlsxColor>()?;
                let format = this.0.set_background_color(background_color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_foreground_color",
            |_, (this, foreground_color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let foreground_color = foreground_color.borrow::<XlsxColor>()?;
                let format = this.0.set_foreground_color(foreground_color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.borrow::<XlsxBorder>()?;
                let format = this.0.set_border(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.borrow::<XlsxColor>()?;
                let format = this.0.set_border_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_top",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.borrow::<XlsxBorder>()?;
                let format = this.0.set_border_top(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_top_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.borrow::<XlsxColor>()?;
                let format = this.0.set_border_top_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_bottom",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.borrow::<XlsxBorder>()?;
                let format = this.0.set_border_bottom(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_bottom_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.borrow::<XlsxColor>()?;
                let format = this.0.set_border_bottom_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_left",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.borrow::<XlsxBorder>()?;
                let format = this.0.set_border_left(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_left_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.borrow::<XlsxColor>()?;
                let format = this.0.set_border_left_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_right",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.borrow::<XlsxBorder>()?;
                let format = this.0.set_border_right(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_right_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.borrow::<XlsxColor>()?;
                let format = this.0.set_border_right_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_diagonal",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.borrow::<XlsxBorder>()?;
                let format = this.0.set_border_diagonal(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_diagonal_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.borrow::<XlsxColor>()?;
                let format = this.0.set_border_diagonal_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_diagonal_type",
            |_, (this, border_type): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border_type = border_type.borrow::<XlsxDiagonalBorder>()?;
                let format = this.0.set_border_diagonal_type(border_type.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function("set_hyperlink", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.set_hyperlink();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("set_unlocked", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.set_unlocked();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("set_hidden", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.set_hidden();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("unset_bold", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.unset_bold();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("unset_italic", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.unset_italic();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("unset_font_strikethrough", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.unset_font_strikethrough();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("unset_text_wrap", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.unset_text_wrap();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("unset_shrink", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.unset_shrink();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("set_locked", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.set_locked();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("unset_hidden", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            let format = this.0.unset_hidden();
            Ok(XlsxFormat(format))
        });
        _methods.add_function("unset_hyperlink_style", |_, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            Ok(XlsxFormat(this.0.unset_hyperlink_style()))
        })
    }
}
