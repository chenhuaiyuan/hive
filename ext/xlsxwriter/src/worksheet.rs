use mlua::prelude::*;
use rust_xlsxwriter::Worksheet;

use crate::{format::XlsxFormat, image::XlsxImage, XlsxColor};
use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};

pub struct XlsxWorksheet(pub Worksheet);

pub fn create_worksheet(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| Ok(XlsxWorksheet(Worksheet::new())))
}

impl LuaUserData for XlsxWorksheet {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method_mut("set_name", |_, this, name: String| {
            this.0.set_name(&name).to_lua_err()?;
            Ok(())
        });
        _methods.add_method("name", |_, this, ()| Ok(this.0.name()));
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
            "write_url",
            |_, this, (row, col, string): (u32, u16, String)| {
                this.0.write_url(row, col, &string).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_url_with_text",
            |_, this, (row, col, string, text): (u32, u16, String, String)| {
                this.0
                    .write_url_with_text(row, col, &string, &text)
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_url_with_format",
            |_, this, (row, col, string, format): (u32, u16, String, LuaAnyUserData)| {
                let format = format.borrow::<XlsxFormat>()?;
                this.0
                    .write_url_with_format(row, col, &string, &format.0)
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut(
            "write_url_with_options",
            |_,
             this,
             (row, col, string, text, tip, format): (
                u32,
                u16,
                String,
                String,
                String,
                Option<LuaAnyUserData>,
            )| {
                if let Some(fmt) = format {
                    let format = fmt.borrow::<XlsxFormat>()?;
                    this.0
                        .write_url_with_options(row, col, &string, &text, &tip, Some(&format.0))
                        .to_lua_err()?;
                } else {
                    this.0
                        .write_url_with_options(row, col, &string, &text, &tip, None)
                        .to_lua_err()?;
                }
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
        _methods.add_method_mut(
            "insert_image",
            |_, this, (row, col, image): (u32, u16, LuaAnyUserData)| {
                let image = image.borrow::<XlsxImage>()?;
                this.0.insert_image(row, col, &image.0).to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_method_mut("insert_image_with_offset", |_, this, (row, col, image, x_offset, y_offset): (u32, u16, LuaAnyUserData, u32, u32)| {
        let image = image.borrow::<XlsxImage>()?;
        this.0.insert_image_with_offset(row, col, &image.0, x_offset, y_offset).to_lua_err()?;
        Ok(())
      });
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
