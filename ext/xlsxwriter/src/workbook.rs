use mlua::prelude::*;
use rust_xlsxwriter::Workbook;

use crate::worksheet::XlsxWorksheet;

pub struct XlsxWorkbook(Workbook);

pub fn create_workbook(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| Ok(XlsxWorkbook(Workbook::new())))
}

impl LuaUserData for XlsxWorkbook {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        // todo
        // _methods.add_method_mut("add_worksheet", |_, this, ()| {
        //     Ok(XlsxWorksheet(this.0.add_worksheet()))
        // });
        // _methods.add_method_mut("worksheet_from_index", |_, this, index: usize| {
        //     let worksheet = this.0.worksheet_from_index(index).to_lua_err()?;
        //     Ok(worksheet)
        // });
        // _methods.add_method_mut("worksheet_from_name", |_, this, sheetname: String| {
        //     let worksheet = this.0.worksheet_from_name(&sheetname).to_lua_err()?;
        //     Ok(worksheet)
        // });
        // todo
        // _methods.add_method_mut("worksheets_mut", |_, this, ()| {

        // });
        // _methods.add_method_mut("worksheets", method)
        _methods.add_method_mut("push_worksheet", |_, this, worksheet: LuaAnyUserData| {
            let worksheet = worksheet.take::<XlsxWorksheet>()?.0;
            this.0.push_worksheet(worksheet);
            Ok(())
        });
        _methods.add_method_mut("save", |_, this, path: String| {
            this.0.save(&path).to_lua_err()?;
            Ok(())
        });
        _methods.add_method_mut("save_to_buffer", |lua, this, ()| {
            let buffer = this.0.save_to_buffer().to_lua_err()?;
            lua.create_string(&buffer)
        });
    }
}
