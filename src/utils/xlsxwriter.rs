use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use mlua::prelude::*;
use rust_xlsxwriter::{
    Format, Workbook, Worksheet, XlsxAlign as Align, XlsxBorder as Border, XlsxColor as Color,
    XlsxDiagonalBorder as DiagonalBorder, XlsxPattern as Pattern, XlsxScript as Script,
    XlsxUnderline as Underline,
};

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
        _methods.add_function("sheet", |_, ()| {
            let sheet = Worksheet::new();
            Ok(XlsxWorksheet(sheet))
        });
        _methods.add_function("format", |_, ()| Ok(XlsxFormat(Format::new())));
        _methods.add_function("color_table", |lua, ()| {
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
        });
        _methods.add_function("underline_table", |lua, ()| {
            lua.create_table_from([
                (
                    "none",
                    lua.create_function(|_, ()| Ok(XlsxUnderline(Underline::None)))?,
                ),
                (
                    "single",
                    lua.create_function(|_, ()| Ok(XlsxUnderline(Underline::Single)))?,
                ),
                (
                    "double",
                    lua.create_function(|_, ()| Ok(XlsxUnderline(Underline::Double)))?,
                ),
                (
                    "single_accounting",
                    lua.create_function(|_, ()| Ok(XlsxUnderline(Underline::SingleAccounting)))?,
                ),
                (
                    "double_accounting",
                    lua.create_function(|_, ()| Ok(XlsxUnderline(Underline::DoubleAccounting)))?,
                ),
            ])
        });
        _methods.add_function("script_table", |lua, ()| {
            lua.create_table_from([
                (
                    "none",
                    lua.create_function(|_, ()| Ok(XlsxScript(Script::None)))?,
                ),
                (
                    "superscript",
                    lua.create_function(|_, ()| Ok(XlsxScript(Script::Superscript)))?,
                ),
                (
                    "subscript",
                    lua.create_function(|_, ()| Ok(XlsxScript(Script::Subscript)))?,
                ),
            ])
        });
        _methods.add_function("align_table", |lua, ()| {
            lua.create_table_from([
                (
                    "general",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::General)))?,
                ),
                (
                    "left",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::Left)))?,
                ),
                (
                    "center",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::Center)))?,
                ),
                (
                    "right",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::Right)))?,
                ),
                (
                    "fill",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::Fill)))?,
                ),
                (
                    "justify",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::Justify)))?,
                ),
                (
                    "center_across",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::CenterAcross)))?,
                ),
                (
                    "distributed",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::Distributed)))?,
                ),
                (
                    "top",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::Top)))?,
                ),
                (
                    "bottom",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::Bottom)))?,
                ),
                (
                    "vertical_center",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::VerticalCenter)))?,
                ),
                (
                    "vertical_justify",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::VerticalJustify)))?,
                ),
                (
                    "vertical_distributed",
                    lua.create_function(|_, ()| Ok(XlsxAlign(Align::VerticalDistributed)))?,
                ),
            ])
        });
        _methods.add_function("pattern_table", |lua, ()| {
            lua.create_table_from([
                (
                    "none",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::None)))?,
                ),
                (
                    "solid",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::Solid)))?,
                ),
                (
                    "medium_gray",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::MediumGray)))?,
                ),
                (
                    "dark_gray",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::DarkGray)))?,
                ),
                (
                    "light_gray",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::LightGray)))?,
                ),
                (
                    "dark_horizontal",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::DarkHorizontal)))?,
                ),
                (
                    "dark_vertical",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::DarkVertical)))?,
                ),
                (
                    "dark_down",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::DarkDown)))?,
                ),
                (
                    "dark_up",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::DarkUp)))?,
                ),
                (
                    "dark_grid",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::DarkGrid)))?,
                ),
                (
                    "dark_trellis",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::DarkTrellis)))?,
                ),
                (
                    "light_horizontal",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::LightHorizontal)))?,
                ),
                (
                    "light_vertical",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::LightVertical)))?,
                ),
                (
                    "light_down",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::LightDown)))?,
                ),
                (
                    "light_up",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::LightUp)))?,
                ),
                (
                    "light_grid",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::LightGrid)))?,
                ),
                (
                    "light_trellis",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::LightTrellis)))?,
                ),
                (
                    "gray125",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::Gray125)))?,
                ),
                (
                    "gray0625",
                    lua.create_function(|_, ()| Ok(XlsxPattern(Pattern::Gray0625)))?,
                ),
            ])
        });
        _methods.add_function("border_table", |lua, ()| {
            lua.create_table_from([
                (
                    "none",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::None)))?,
                ),
                (
                    "thin",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::Thin)))?,
                ),
                (
                    "medium",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::Medium)))?,
                ),
                (
                    "dashed",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::Dashed)))?,
                ),
                (
                    "dotted",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::Dotted)))?,
                ),
                (
                    "thick",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::Thick)))?,
                ),
                (
                    "double",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::Double)))?,
                ),
                (
                    "hair",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::Hair)))?,
                ),
                (
                    "medium_dashed",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::MediumDashed)))?,
                ),
                (
                    "dash_dot",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::DashDot)))?,
                ),
                (
                    "medium_dash_dot",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::MediumDashDot)))?,
                ),
                (
                    "dash_dot_dot",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::DashDotDot)))?,
                ),
                (
                    "medium_dash_dot_dot",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::MediumDashDotDot)))?,
                ),
                (
                    "slant_dash_dot",
                    lua.create_function(|_, ()| Ok(XlsxBorder(Border::SlantDashDot)))?,
                ),
            ])
        });
        _methods.add_function("diagonal_border", |lua, ()| {
            lua.create_table_from([
                (
                    "none",
                    lua.create_function(|_, ()| Ok(XlsxDiagonalBorder(DiagonalBorder::None)))?,
                ),
                (
                    "border_up",
                    lua.create_function(|_, ()| Ok(XlsxDiagonalBorder(DiagonalBorder::BorderUp)))?,
                ),
                (
                    "border_down",
                    lua.create_function(|_, ()| {
                        Ok(XlsxDiagonalBorder(DiagonalBorder::BorderDown))
                    })?,
                ),
                (
                    "border_up_down",
                    lua.create_function(|_, ()| {
                        Ok(XlsxDiagonalBorder(DiagonalBorder::BorderUpDown))
                    })?,
                ),
            ])
        });
    }
}

pub struct XlsxWorksheet(Worksheet);

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
        _methods.add_method_mut(
            "set_header_footer_scale_with_doc",
            |_, this, enable: bool| {
                this.0.set_header_footer_scale_with_doc(enable);
                Ok(())
            },
        );
        _methods.add_method_mut(
            "set_header_footer_align_with_page",
            |_, this, enable: bool| {
                this.0.set_header_footer_align_with_page(enable);
                Ok(())
            },
        );
        _methods.add_method_mut("set_margins", |_, this, (left, right, top, bottom, header, footer): (f64, f64, f64, f64, f64, f64)| {
          this.0.set_margins(left, right, top, bottom, header, footer);
          Ok(())
        });
        _methods.add_method_mut(
            "set_print_first_page_number",
            |_, this, page_number: u16| {
                this.0.set_print_first_page_number(page_number);
                Ok(())
            },
        );
        _methods.add_method_mut("set_print_scale", |_, this, scale: u16| {
            this.0.set_print_scale(scale);
            Ok(())
        });
        _methods.add_method_mut(
            "set_print_fit_to_pages",
            |_, this, (width, height): (u16, u16)| {
                this.0.set_print_fit_to_pages(width, height);
                Ok(())
            },
        );
        _methods.add_method_mut("set_print_center_horizontally", |_, this, enable: bool| {
            this.0.set_print_center_horizontally(enable);
            Ok(())
        });
        _methods.add_method_mut("set_print_center_vertically", |_, this, enable: bool| {
            this.0.set_print_center_vertically(enable);
            Ok(())
        });
        _methods.add_method_mut("set_print_gridlines", |_, this, enable: bool| {
            this.0.set_print_gridlines(enable);
            Ok(())
        });
        _methods.add_method_mut("set_print_black_and_white", |_, this, enable: bool| {
            this.0.set_print_black_and_white(enable);
            Ok(())
        });
        _methods.add_method_mut("set_print_draft", |_, this, enable: bool| {
            this.0.set_print_draft(enable);
            Ok(())
        });
        _methods.add_method_mut("set_print_headings", |_, this, enable: bool| {
            this.0.set_print_headings(enable);
            Ok(())
        });
        _methods.add_method_mut(
            "set_print_area",
            |_, this, (first_row, first_col, last_row, last_col): (u32, u16, u32, u16)| {
                this.0
                    .set_print_area(first_row, first_col, last_row, last_col)
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "set_repeat_rows",
            |_, this, (first_row, last_row): (u32, u32)| {
                this.0.set_repeat_rows(first_row, last_row).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "set_repeat_columns",
            |_, this, (first_col, last_col): (u16, u16)| {
                this.0
                    .set_repeat_columns(first_col, last_col)
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut("set_autofit", |_, this, ()| {
            this.0.set_autofit();
            Ok(())
        });
    }
}

pub struct XlsxFormat(Format);

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
                let color = font_color.take::<XlsxColor>()?;
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
                let underline = underline.take::<XlsxUnderline>()?;
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
                let script = font_script.take::<XlsxScript>()?;
                let format = this.0.set_font_script(script.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_align",
            |_, (this, align): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let align = align.take::<XlsxAlign>()?;
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
                let pattern = pattern.take::<XlsxPattern>()?;
                let format = this.0.set_pattern(pattern.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_background_color",
            |_, (this, background_color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let background_color = background_color.take::<XlsxColor>()?;
                let format = this.0.set_background_color(background_color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_foreground_color",
            |_, (this, foreground_color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let foreground_color = foreground_color.take::<XlsxColor>()?;
                let format = this.0.set_foreground_color(foreground_color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.take::<XlsxBorder>()?;
                let format = this.0.set_border(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.take::<XlsxColor>()?;
                let format = this.0.set_border_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_top",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.take::<XlsxBorder>()?;
                let format = this.0.set_border_top(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_top_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.take::<XlsxColor>()?;
                let format = this.0.set_border_top_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_bottom",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.take::<XlsxBorder>()?;
                let format = this.0.set_border_bottom(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_bottom_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.take::<XlsxColor>()?;
                let format = this.0.set_border_bottom_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_left",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.take::<XlsxBorder>()?;
                let format = this.0.set_border_left(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_left_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.take::<XlsxColor>()?;
                let format = this.0.set_border_left_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_right",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.take::<XlsxBorder>()?;
                let format = this.0.set_border_right(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_right_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.take::<XlsxColor>()?;
                let format = this.0.set_border_right_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_diagonal",
            |_, (this, border): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border = border.take::<XlsxBorder>()?;
                let format = this.0.set_border_diagonal(border.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_diagonal_color",
            |_, (this, color): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let color = color.take::<XlsxColor>()?;
                let format = this.0.set_border_diagonal_color(color.0);
                Ok(XlsxFormat(format))
            },
        );
        _methods.add_function(
            "set_border_diagonal_type",
            |_, (this, border_type): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let border_type = border_type.take::<XlsxDiagonalBorder>()?;
                let format = this.0.set_border_diagonal_type(border_type.0);
                Ok(XlsxFormat(format))
            },
        );
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
    }
}

pub struct XlsxColor(Color);

impl LuaUserData for XlsxColor {}

pub struct XlsxUnderline(Underline);

impl LuaUserData for XlsxUnderline {}

pub struct XlsxScript(Script);

impl LuaUserData for XlsxScript {}

pub struct XlsxAlign(Align);

impl LuaUserData for XlsxAlign {}

pub struct XlsxPattern(Pattern);

impl LuaUserData for XlsxPattern {}

pub struct XlsxBorder(Border);

impl LuaUserData for XlsxBorder {}

pub struct XlsxDiagonalBorder(DiagonalBorder);

impl LuaUserData for XlsxDiagonalBorder {}
