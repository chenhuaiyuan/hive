use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;

use crate::error::Error as WebError;
use mlua::prelude::*;
use nanoid::nanoid;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct File {
    field_name: String,
    file_name: String,
    content_type: String,
    pub content: Vec<u8>,
}

impl File {
    pub fn new<S: Into<String>>(
        field_name: S,
        file_name: S,
        content_type: S,
        content: Vec<u8>,
    ) -> Self {
        Self {
            field_name: field_name.into(),
            file_name: file_name.into(),
            content_type: content_type.into(),
            content,
        }
    }
}

impl LuaUserData for File {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {
        _fields.add_field_method_get("field_name", |lua, this| {
            lua.create_string(&this.field_name)
        });
        _fields.add_field_method_get("file_name", |lua, this| lua.create_string(&this.file_name));
        _fields.add_field_method_get("content_type", |lua, this| {
            lua.create_string(&this.content_type)
        });
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_async_function(
            "save",
            |lua, (this, path): (LuaAnyUserData, LuaMultiValue)| async move {
                let alphabet = [
                    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
                    'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
                    'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
                    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
                ];
                let this = this.take::<Self>()?;
                if path.is_empty() {
                    let file: Vec<&str> = this.file_name.split('.').collect();
                    if file.len() == 2 {
                        let suffix = file[1];

                        let random_name = nanoid!(16, &alphabet) + ".";

                        let new_file_name = random_name + suffix;
                        let mut file =
                            fs::File::create(new_file_name.clone()).await.to_lua_err()?;
                        file.write_all(&this.content).await.to_lua_err()?;
                        // fs::write(new_file_name, this.content.as_ref())
                        //     .await
                        //     .to_lua_err()?;
                        let file_name = lua.create_string(&new_file_name)?;
                        return Ok((true, file_name));
                    }
                    return Ok((false, lua.create_string(&"")?));
                }
                let path = path.into_vec();
                if path.len() == 1 {
                    match path[0].clone() {
                        LuaValue::String(v) => {
                            let mut p = v.to_str()?.to_string();
                            let last_char = p.pop().unwrap_or('/');
                            if last_char == '/' {
                                p.push('/');
                            } else {
                                p.push(last_char);
                                p.push('/');
                            }

                            let dir = Path::new(&p);

                            if !dir.exists() {
                                fs::create_dir_all(dir).await.to_lua_err()?;
                            }

                            let file: Vec<&str> = this.file_name.split('.').collect();
                            let suffix = file[1];

                            let random_name = nanoid!(16, &alphabet) + ".";

                            let new_file_name = dir.join(random_name + suffix);
                            let mut file =
                                fs::File::create(new_file_name.clone()).await.to_lua_err()?;
                            file.write_all(&this.content).await.to_lua_err()?;
                            // fs::write(new_file_name, this.content.as_ref())
                            //     .await
                            //     .to_lua_err()?;
                            let f_name = new_file_name.to_str().unwrap_or("");
                            let file_name = lua.create_string(&f_name)?;
                            return Ok((true, file_name));
                        }
                        _ => {
                            return Ok((false, lua.create_string(&"")?));
                        }
                    }
                } else if path.len() >= 2 {
                    match path[0].clone() {
                        LuaValue::String(v) => {
                            let mut p = v.to_str()?.to_string();
                            let last_char = p.pop().unwrap_or('/');
                            if last_char == '/' {
                                p.push('/');
                            } else {
                                p.push(last_char);
                                p.push('/');
                            }

                            let dir = Path::new(&p);

                            if !dir.exists() {
                                fs::create_dir_all(dir).await.to_lua_err()?;
                            }

                            match path[1].clone() {
                                LuaValue::String(f) => {
                                    let file_name = f.to_str()?;

                                    let new_file_name = dir.join(file_name);
                                    let mut file = fs::File::create(new_file_name.clone())
                                        .await
                                        .to_lua_err()?;
                                    file.write_all(&this.content).await.to_lua_err()?;
                                    // fs::write(new_file_name, this.content.as_ref())
                                    //     .await
                                    //     .to_lua_err()?;
                                    let f_name = new_file_name.to_str().unwrap_or("");
                                    let file_name = lua.create_string(&f_name)?;
                                    return Ok((true, file_name));
                                }
                                _ => {
                                    return Ok((false, lua.create_string(&"")?));
                                }
                            }
                        }
                        _ => {
                            return Ok((false, lua.create_string(&"")?));
                        }
                    }
                }
                Ok((false, lua.create_string(&"")?))
            },
        );
        _methods.add_async_function("new", |_, file_path: String| async move {
            let path = Path::new(&file_path);

            if path.exists() {
                if path.is_file() {
                    let file_name = path
                        .file_name()
                        .unwrap_or_else(|| OsStr::new("default.txt"))
                        .to_str()
                        .unwrap_or("default.txt");
                    let field_name: Vec<&str> = file_name.split('.').collect();
                    let ext = path
                        .extension()
                        .unwrap_or_else(|| OsStr::new("txt"))
                        .to_str()
                        .unwrap_or("txt");

                    let mut f = fs::File::open(path).await.to_lua_err()?;
                    let mut buffer = Vec::new();
                    f.read_to_end(&mut buffer).await?;
                    let file = File::new(field_name[0], file_name, ext, buffer);
                    Ok(file)
                } else {
                    Err(LuaError::ExternalError(Arc::new(WebError::new(
                        4031,
                        format!("{} must be a file", file_path),
                    ))))
                }
            } else {
                Err(LuaError::ExternalError(Arc::new(WebError::new(
                    4030,
                    format!("{} Not Found", file_path),
                ))))
            }
        });
        _methods.add_async_function("get_file", |lua, this: LuaAnyUserData| async move {
            let table = lua.create_table()?;
            let headers = lua.create_table()?;
            let this = this.take::<Self>()?;
            table.set("status", LuaValue::Integer(200))?;
            if this.content_type == "txt" {
                let s = lua.create_string(&"text/plain")?;
                headers.set("Content-Type", LuaValue::String(s))?;
            } else if this.content_type == "html" {
                headers.set(
                    "Content-Type",
                    LuaValue::String(lua.create_string(&"text/html")?),
                )?;
            } else if this.content_type == "xml" {
                // headers.set("Content-Type", Lua.create_string("text/xml")?)?;
                headers.set(
                    "Content-Type",
                    LuaValue::String(lua.create_string("application/xml")?),
                )?;
            } else if this.content_type == "gif" {
                headers.set(
                    "Content-Type",
                    LuaValue::String(lua.create_string("image/gif")?),
                )?;
            } else if this.content_type == "jpeg" || this.content_type == "jpg" {
                headers.set(
                    "Content-Type",
                    LuaValue::String(lua.create_string("image/jpeg")?),
                )?;
            } else if this.content_type == "png" {
                headers.set(
                    "Content-Type",
                    LuaValue::String(lua.create_string("image/png")?),
                )?;
            } else if this.content_type == "xhtml" {
                headers.set(
                    "Content-Type",
                    LuaValue::String(lua.create_string("application/xhtml+xml")?),
                )?;
            } else if this.content_type == "json" {
                headers.set(
                    "Content-Type",
                    LuaValue::String(lua.create_string("application/json")?),
                )?;
            } else if this.content_type == "pdf" {
                headers.set(
                    "Content-Type",
                    LuaValue::String(lua.create_string("application/pdf")?),
                )?;
            } else if this.content_type == "docx" {
                headers.set(
                    "Content-Type",
                    LuaValue::String(lua.create_string("application/msword")?),
                )?;
            } else {
                headers.set(
                    "Content-Type",
                    LuaValue::String(lua.create_string("application/octet-stream")?),
                )?;
            }
            table.set("headers", headers)?;
            table.set("body", LuaValue::String(lua.create_string(&this.content)?))?;
            Ok(table)
        });
        _methods.add_async_function("download", |lua, this: LuaAnyUserData| async move {
            let table = lua.create_table()?;
            let headers = lua.create_table()?;
            let this = this.take::<Self>()?;
            table.set("status", LuaValue::Integer(200))?;
            headers.set(
                "Content-Type",
                LuaValue::String(lua.create_string("application/octet-stream")?),
            )?;
            headers.set(
                "Content-Disposition",
                LuaValue::String(
                    lua.create_string(&format!(r#"attachment; filename="{}""#, this.file_name))?,
                ),
            )?;
            table.set("headers", headers)?;
            table.set("body", LuaValue::String(lua.create_string(&this.content)?))?;
            Ok(table)
        });
    }
}
