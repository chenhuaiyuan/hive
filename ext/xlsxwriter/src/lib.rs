mod format;
mod image;
mod workbook;
mod worksheet;
use format::create_format;
use image::create_image;
use mlua::prelude::*;
use rust_xlsxwriter::{
    XlsxAlign as Align, XlsxBorder as Border, XlsxColor as Color,
    XlsxDiagonalBorder as DiagonalBorder, XlsxPattern as Pattern, XlsxScript as Script,
    XlsxUnderline as Underline,
};
use workbook::create_workbook;
use worksheet::create_worksheet;

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

#[mlua::lua_module]
pub fn xlsxwriter(lua: &Lua) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    table.set("workbook", create_workbook(lua)?)?;
    table.set("worksheet", create_worksheet(lua)?)?;
    table.set("format", create_format(lua)?)?;
    table.set("image", create_image(lua)?)?;
    table.set(
        "align",
        lua.create_table_from([
            ("general", lua.create_userdata(XlsxAlign(Align::General))?),
            ("left", lua.create_userdata(XlsxAlign(Align::Left))?),
            ("center", lua.create_userdata(XlsxAlign(Align::Center))?),
            ("right", lua.create_userdata(XlsxAlign(Align::Right))?),
            ("fill", lua.create_userdata(XlsxAlign(Align::Fill))?),
            ("justify", lua.create_userdata(XlsxAlign(Align::Justify))?),
            (
                "center_across",
                lua.create_userdata(XlsxAlign(Align::CenterAcross))?,
            ),
            (
                "distributed",
                lua.create_userdata(XlsxAlign(Align::Distributed))?,
            ),
            ("top", lua.create_userdata(XlsxAlign(Align::Top))?),
            ("bottom", lua.create_userdata(XlsxAlign(Align::Bottom))?),
            (
                "vertical_center",
                lua.create_userdata(XlsxAlign(Align::VerticalCenter))?,
            ),
            (
                "vertical_justify",
                lua.create_userdata(XlsxAlign(Align::VerticalJustify))?,
            ),
            (
                "vertical_distributed",
                lua.create_userdata(XlsxAlign(Align::VerticalDistributed))?,
            ),
        ])?,
    )?;
    table.set(
        "border",
        lua.create_table_from([
            ("none", lua.create_userdata(XlsxBorder(Border::None))?),
            ("thin", lua.create_userdata(XlsxBorder(Border::Thin))?),
            ("medium", lua.create_userdata(XlsxBorder(Border::Medium))?),
            ("dashed", lua.create_userdata(XlsxBorder(Border::Dashed))?),
            ("dotted", lua.create_userdata(XlsxBorder(Border::Dotted))?),
            ("thick", lua.create_userdata(XlsxBorder(Border::Thick))?),
            ("double", lua.create_userdata(XlsxBorder(Border::Double))?),
            ("hair", lua.create_userdata(XlsxBorder(Border::Hair))?),
            (
                "medium_dashed",
                lua.create_userdata(XlsxBorder(Border::MediumDashed))?,
            ),
            (
                "dash_dot",
                lua.create_userdata(XlsxBorder(Border::DashDot))?,
            ),
            (
                "medium_dash_dot",
                lua.create_userdata(XlsxBorder(Border::MediumDashDot))?,
            ),
            (
                "dash_dot_dot",
                lua.create_userdata(XlsxBorder(Border::DashDotDot))?,
            ),
            (
                "medium_dash_dot_dot",
                lua.create_userdata(XlsxBorder(Border::MediumDashDotDot))?,
            ),
            (
                "slant_dash_dot",
                lua.create_userdata(XlsxBorder(Border::SlantDashDot))?,
            ),
        ])?,
    )?;
    let color = lua.create_table()?;
    color.set(
        "rgb",
        lua.create_function(|_, rgb: u32| Ok(XlsxColor(Color::RGB(rgb))))?,
    )?;
    color.set(
        "theme",
        lua.create_function(|_, (a, b): (u8, u8)| Ok(XlsxColor(Color::Theme(a, b))))?,
    )?;
    color.set(
        "automatic",
        lua.create_userdata(XlsxColor(Color::Automatic))?,
    )?;
    color.set("black", lua.create_userdata(XlsxColor(Color::Black))?)?;
    color.set("blue", lua.create_userdata(XlsxColor(Color::Blue))?)?;
    color.set("brown", lua.create_userdata(XlsxColor(Color::Brown))?)?;
    color.set("cyan", lua.create_userdata(XlsxColor(Color::Cyan))?)?;
    color.set("gray", lua.create_userdata(XlsxColor(Color::Gray))?)?;
    color.set("green", lua.create_userdata(XlsxColor(Color::Green))?)?;
    color.set("lime", lua.create_userdata(XlsxColor(Color::Lime))?)?;
    color.set("magenta", lua.create_userdata(XlsxColor(Color::Magenta))?)?;
    color.set("navy", lua.create_userdata(XlsxColor(Color::Navy))?)?;
    color.set("orange", lua.create_userdata(XlsxColor(Color::Orange))?)?;
    color.set("pink", lua.create_userdata(XlsxColor(Color::Pink))?)?;
    color.set("purple", lua.create_userdata(XlsxColor(Color::Purple))?)?;
    color.set("red", lua.create_userdata(XlsxColor(Color::Red))?)?;
    color.set("silver", lua.create_userdata(XlsxColor(Color::Silver))?)?;
    color.set("white", lua.create_userdata(XlsxColor(Color::White))?)?;
    color.set("yellow", lua.create_userdata(XlsxColor(Color::Yellow))?)?;
    table.set("color", color)?;
    table.set(
        "diagonal_border",
        lua.create_table_from([
            (
                "none",
                lua.create_userdata(XlsxDiagonalBorder(DiagonalBorder::None))?,
            ),
            (
                "border_up",
                lua.create_userdata(XlsxDiagonalBorder(DiagonalBorder::BorderUp))?,
            ),
            (
                "border_down",
                lua.create_userdata(XlsxDiagonalBorder(DiagonalBorder::BorderDown))?,
            ),
            (
                "border_up_down",
                lua.create_userdata(XlsxDiagonalBorder(DiagonalBorder::BorderUpDown))?,
            ),
        ])?,
    )?;
    table.set(
        "pattern",
        lua.create_table_from([
            ("none", lua.create_userdata(XlsxPattern(Pattern::None))?),
            ("solid", lua.create_userdata(XlsxPattern(Pattern::Solid))?),
            (
                "medium_gray",
                lua.create_userdata(XlsxPattern(Pattern::MediumGray))?,
            ),
            (
                "dark_gray",
                lua.create_userdata(XlsxPattern(Pattern::DarkGray))?,
            ),
            (
                "light_gray",
                lua.create_userdata(XlsxPattern(Pattern::LightGray))?,
            ),
            (
                "dark_horizontal",
                lua.create_userdata(XlsxPattern(Pattern::DarkHorizontal))?,
            ),
            (
                "dark_vertical",
                lua.create_userdata(XlsxPattern(Pattern::DarkVertical))?,
            ),
            (
                "dark_down",
                lua.create_userdata(XlsxPattern(Pattern::DarkDown))?,
            ),
            (
                "dark_up",
                lua.create_userdata(XlsxPattern(Pattern::DarkUp))?,
            ),
            (
                "dark_grid",
                lua.create_userdata(XlsxPattern(Pattern::DarkGrid))?,
            ),
            (
                "dark_trellis",
                lua.create_userdata(XlsxPattern(Pattern::DarkTrellis))?,
            ),
            (
                "light_horizontal",
                lua.create_userdata(XlsxPattern(Pattern::LightHorizontal))?,
            ),
            (
                "light_vertical",
                lua.create_userdata(XlsxPattern(Pattern::LightVertical))?,
            ),
            (
                "light_down",
                lua.create_userdata(XlsxPattern(Pattern::LightDown))?,
            ),
            (
                "light_up",
                lua.create_userdata(XlsxPattern(Pattern::LightUp))?,
            ),
            (
                "light_grid",
                lua.create_userdata(XlsxPattern(Pattern::LightGrid))?,
            ),
            (
                "light_trellis",
                lua.create_userdata(XlsxPattern(Pattern::LightTrellis))?,
            ),
            (
                "gray125",
                lua.create_userdata(XlsxPattern(Pattern::Gray125))?,
            ),
            (
                "gray0625",
                lua.create_userdata(XlsxPattern(Pattern::Gray0625))?,
            ),
        ])?,
    )?;
    table.set(
        "script",
        lua.create_table_from([
            ("none", lua.create_userdata(XlsxScript(Script::None))?),
            (
                "superscript",
                lua.create_userdata(XlsxScript(Script::Superscript))?,
            ),
            (
                "subscript",
                lua.create_userdata(XlsxScript(Script::Subscript))?,
            ),
        ])?,
    )?;
    table.set(
        "underline",
        lua.create_table_from([
            ("none", lua.create_userdata(XlsxUnderline(Underline::None))?),
            (
                "single",
                lua.create_userdata(XlsxUnderline(Underline::Single))?,
            ),
            (
                "double",
                lua.create_userdata(XlsxUnderline(Underline::Double))?,
            ),
            (
                "single_accounting",
                lua.create_userdata(XlsxUnderline(Underline::SingleAccounting))?,
            ),
            (
                "double_accounting",
                lua.create_userdata(XlsxUnderline(Underline::DoubleAccounting))?,
            ),
        ])?,
    )?;
    Ok(table)
}
