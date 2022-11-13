use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use mlua::prelude::*;
use rust_xlsxwriter::{Format, Workbook, Worksheet, XlsxColor as Color};

pub struct XlsxWorkbook(Workbook);

pub fn create_xlsx_book(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| Ok(XlsxWorkbook(Workbook::new())))
}

impl LuaUserData for XlsxWorkbook {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method_mut("push_worksheet", |_, this, worksheet: LuaAnyUserData| {
            let sheet = worksheet.take::<XlsxWorksheet>()?;
            this.0.push_worksheet(sheet.0);
            Ok(())
        });
        _methods.add_method_mut("save", |_, this, file_name: String| {
            this.0.save(&file_name).to_lua_err()?;
            Ok(())
        });
        _methods.add_method_mut("save_to_buffer", |lua, this, ()| {
            let buffer = this.0.save_to_buffer().to_lua_err()?;
            lua.create_string(&buffer)
        });
    }
}

pub struct XlsxWorksheet(Worksheet);

pub fn create_xlsx_sheet(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| {
        let sheet = Worksheet::new();
        Ok(XlsxWorksheet(sheet))
    })
}

impl LuaUserData for XlsxWorksheet {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method_mut("set_name", |_, this, name: String| {
            this.0.set_name(&name).to_lua_err()?;
            Ok(())
        });
        _methods.add_method_mut(
            "write_number",
            |_, this, (row, col, number, format): (u32, u16, f64, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0
                    .write_number(row, col, number, &(format.0))
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_number_only",
            |_, this, (row, col, number): (u32, u16, f64)| {
                this.0.write_number_only(row, col, number).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_string",
            |_, this, (row, col, string, format): (u32, u16, String, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0
                    .write_string(row, col, &string, &(format.0))
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_string_only",
            |_, this, (row, col, string): (u32, u16, String)| {
                this.0.write_string_only(row, col, &string).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_formula",
            |_, this, (row, col, formula, format): (u32, u16, String, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0
                    .write_formula(row, col, &formula, &(format.0))
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_formula_only",
            |_, this, (row, col, formula): (u32, u16, String)| {
                this.0.write_formula_only(row, col, &formula).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_array_formula",
            |_,
             this,
             (first_row, first_col, last_row, last_col, formula, format): (
                u32,
                u16,
                u32,
                u16,
                String,
                LuaAnyUserData,
            )| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0
                    .write_array_formula(
                        first_row,
                        first_col,
                        last_row,
                        last_col,
                        &formula,
                        &(format.0),
                    )
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut("write_array_formula_only", |_, this, (first_row, first_col, last_row, last_col, formula): (u32, u16, u32, u16, String)| {
          this.0.write_array_formula_only(first_row, first_col, last_row, last_col, &formula).to_lua_err()?;
          Ok(())
        });
        _methods.add_method_mut(
            "write_dynamic_array_formula",
            |_,
             this,
             (first_row, first_col, last_row, last_col, formula, format): (
                u32,
                u16,
                u32,
                u16,
                String,
                LuaAnyUserData,
            )| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0
                    .write_dynamic_array_formula(
                        first_row,
                        first_col,
                        last_row,
                        last_col,
                        &formula,
                        &(format.0),
                    )
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut("write_dynamic_array_formula_only", |_, this, (first_row, first_col, last_row, last_col, formula): (u32, u16, u32, u16, String)| {
          this.0.write_dynamic_array_formula_only(first_row, first_col, last_row, last_col, &formula).to_lua_err()?;
          Ok(())
        });
        _methods.add_method_mut(
            "write_dynamic_formula",
            |_, this, (row, col, formula, format): (u32, u16, String, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0
                    .write_dynamic_formula(row, col, &formula, &(format.0))
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_dynamic_formula_only",
            |_, this, (row, col, formula): (u32, u16, String)| {
                this.0
                    .write_dynamic_formula_only(row, col, &formula)
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_blank",
            |_, this, (row, col, format): (u32, u16, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0.write_blank(row, col, &(format.0)).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_datetime",
            |_, this, (row, col, datetime, format): (u32, u16, String, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                let datetime = datetime.parse::<NaiveDateTime>().to_lua_err()?;
                this.0
                    .write_datetime(row, col, datetime, &(format.0))
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_date",
            |_, this, (row, col, date, format): (u32, u16, String, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                let date = date.parse::<NaiveDate>().to_lua_err()?;
                this.0
                    .write_date(row, col, date, &(format.0))
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_time",
            |_, this, (row, col, time, format): (u32, u16, String, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                let time = time.parse::<NaiveTime>().to_lua_err()?;
                this.0
                    .write_time(row, col, time, &(format.0))
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_boolean",
            |_, this, (row, col, boolean, format): (u32, u16, bool, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0
                    .write_boolean(row, col, boolean, &(format.0))
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_boolean_only",
            |_, this, (row, col, boolean): (u32, u16, bool)| {
                this.0.write_boolean_only(row, col, boolean).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "merge_range",
            |_,
             this,
             (first_row, first_col, last_row, last_col, string, format): (
                u32,
                u16,
                u32,
                u16,
                String,
                LuaAnyUserData,
            )| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0
                    .merge_range(
                        first_row,
                        first_col,
                        last_row,
                        last_col,
                        &string,
                        &(format.0),
                    )
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut("set_row_height", |_, this, (row, height): (u32, f64)| {
            this.0.set_row_height(row, height).to_lua_err()?;
            Ok(())
        });
        _methods.add_method_mut(
            "set_row_height_pixels",
            |_, this, (row, height): (u32, u16)| {
                this.0.set_row_height_pixels(row, height).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "set_row_format",
            |_, this, (row, format): (u32, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0.set_row_format(row, &(format.0)).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut("set_column_width", |_, this, (col, width): (u16, f64)| {
            this.0.set_column_width(col, width).to_lua_err()?;
            Ok(())
        });
        _methods.add_method_mut(
            "set_column_width_pixels",
            |_, this, (col, width): (u16, u16)| {
                this.0.set_column_width_pixels(col, width).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "set_column_format",
            |_, this, (col, format): (u16, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0.set_column_format(col, &(format.0)).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "set_formula_result",
            |_, this, (row, col, result): (u32, u16, String)| {
                this.0.set_formula_result(row, col, &result);
                Ok(())
            },
        );
        _methods.add_method_mut("set_formula_result_default", |_, this, result: String| {
            this.0.set_formula_result_default(&result);
            Ok(())
        });
        _methods.add_method_mut("use_future_functions", |_, this, enable: bool| {
            this.0.use_future_functions(enable);
            Ok(())
        });
        _methods.add_method_mut("set_right_to_left", |_, this, enable: bool| {
            this.0.set_right_to_left(enable);
            Ok(())
        });
        _methods.add_method_mut("set_active", |_, this, enable: bool| {
            this.0.set_active(enable);
            Ok(())
        });
        _methods.add_method_mut("set_selected", |_, this, enable: bool| {
            this.0.set_selected(enable);
            Ok(())
        });
        _methods.add_method_mut("set_hidden", |_, this, enable: bool| {
            this.0.set_hidden(enable);
            Ok(())
        });
        _methods.add_method_mut("set_first_tab", |_, this, enable: bool| {
            this.0.set_first_tab(enable);
            Ok(())
        });
        _methods.add_method_mut("set_tab_color", |_, this, color: LuaAnyUserData| {
            let color = color.borrow::<XlsxColor>()?;
            this.0.set_tab_color(color.0);
            Ok(())
        });
        _methods.add_method_mut("set_paper_size", |_, this, paper_size: u8| {
            this.0.set_paper_size(paper_size);
            Ok(())
        });
        _methods.add_method_mut("set_page_order", |_, this, enable: bool| {
            this.0.set_page_order(enable);
            Ok(())
        });
        _methods.add_method_mut("set_landscape", |_, this, ()| {
            this.0.set_landscape();
            Ok(())
        });
        _methods.add_method_mut("set_portrait", |_, this, ()| {
            this.0.set_portrait();
            Ok(())
        });
        _methods.add_method_mut("set_view_normal", |_, this, ()| {
            this.0.set_view_normal();
            Ok(())
        });
        _methods.add_method_mut("set_view_page_layout", |_, this, ()| {
            this.0.set_view_page_layout();
            Ok(())
        });
        _methods.add_method_mut("set_view_page_break_preview", |_, this, ()| {
            this.0.set_view_page_break_preview();
            Ok(())
        });
        _methods.add_method_mut("set_zoom", |_, this, zoom: u16| {
            this.0.set_zoom(zoom);
            Ok(())
        });
        _methods.add_method_mut("set_freeze_panes", |_, this, (row, col): (u32, u16)| {
            this.0.set_freeze_panes(row, col).to_lua_err()?;
            Ok(())
        });
        _methods.add_method_mut(
            "set_freeze_panes_top_cell",
            |_, this, (row, col): (u32, u16)| {
                this.0.set_freeze_panes_top_cell(row, col).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut("set_header", |_, this, header: String| {
            this.0.set_header(&header);
            Ok(())
        });
        _methods.add_method_mut("set_footer", |_, this, footer: String| {
            this.0.set_footer(&footer);
            Ok(())
        });
    }
}

pub struct XlsxFormat(Format);

pub fn create_xlsx_format(lua: &Lua) -> LuaResult<LuaFunction> {
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
    }
}

pub struct XlsxColor(Color);

pub fn create_color_table(lua: &Lua) -> LuaResult<LuaTable> {
    lua.create_table_from([
        (
            "rgb",
            lua.create_function(|_, rgb: u32| Ok(XlsxColor(Color::RGB(rgb))))?,
        ),
        (
            "theme",
            lua.create_function(|_, (a, b): (u8, u8)| Ok(XlsxColor(Color::Theme(a, b))))?,
        ),
        (
            "automatic",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Automatic)))?,
        ),
        (
            "black",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Black)))?,
        ),
        (
            "blue",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Blue)))?,
        ),
        (
            "brown",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Brown)))?,
        ),
        (
            "cyan",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Cyan)))?,
        ),
        (
            "gray",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Gray)))?,
        ),
        (
            "green",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Green)))?,
        ),
        (
            "lime",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Lime)))?,
        ),
        (
            "magenta",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Magenta)))?,
        ),
        (
            "navy",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Navy)))?,
        ),
        (
            "orange",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Orange)))?,
        ),
        (
            "pink",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Pink)))?,
        ),
        (
            "purple",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Purple)))?,
        ),
        (
            "red",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Red)))?,
        ),
        (
            "silver",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Silver)))?,
        ),
        (
            "white",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::White)))?,
        ),
        (
            "yellow",
            lua.create_function(|_, ()| Ok(XlsxColor(Color::Yellow)))?,
        ),
    ])
}

impl LuaUserData for XlsxColor {}
