// use crate::file_data::FileDataTrait;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Arc;

use crate::error::Error as WebError;
use crate::lua::response::HiveResponse;
use http::Response;
use hyper::Body;
use mlua::prelude::*;
use nanoid::nanoid;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct FileData {
    field_name: String,
    file_name: String,
    content_type: String,
    pub content: Vec<u8>,
}

impl FileData {
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

// impl FileDataTrait for FileData {}

#[cfg(any(
    feature = "lua51",
    feature = "lua52",
    feature = "lua53",
    feature = "lua54",
    feature = "luau",
    feature = "luajit"
))]
impl LuaUserData for FileData {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {
        _fields.add_field_method_get("field_name", |_, this| Ok(this.field_name.clone()));
        _fields.add_field_method_get("file_name", |_, this| Ok(this.file_name.clone()));
        _fields.add_field_method_get("content_type", |_, this| Ok(this.content_type.clone()));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_async_function(
            "save",
            |_, (this, path): (LuaAnyUserData, LuaMultiValue)| async move {
                let alphabet = [
                    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
                    'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
                    'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
                    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
                ];
                let this: FileData = this.take::<Self>()?;
                if path.is_empty() {
                    let file: Vec<&str> = this.file_name.split('.').collect();
                    if file.len() == 2 {
                        let suffix: &str = file[1];

                        let random_name: String = nanoid!(16, &alphabet) + ".";

                        let new_file_name: String = random_name + suffix;
                        let mut file: fs::File =
                            fs::File::create(&new_file_name).await.to_lua_err()?;
                        file.write_all(&this.content).await.to_lua_err()?;
                        return Ok((true, new_file_name));
                    }
                    return Ok((false, "".to_string()));
                }
                let path: Vec<LuaValue> = path.into_vec();
                let path_len = path.len();
                if path_len == 1 {
                    if let LuaValue::String(v) = &path[0] {
                        let mut p: String = v.to_str()?.to_string();
                        let last_char: char = p.pop().unwrap_or('/');
                        if last_char == '/' {
                            p.push('/');
                        } else {
                            p.push(last_char);
                            p.push('/');
                        }

                        let dir: &Path = Path::new(&p);

                        if !dir.exists() {
                            fs::create_dir_all(dir).await.to_lua_err()?;
                        }

                        let file: Vec<&str> = this.file_name.split('.').collect();
                        let suffix: &str = file[1];

                        let random_name: String = nanoid!(16, &alphabet) + ".";

                        let new_file_name: std::path::PathBuf = dir.join(random_name + suffix);
                        let mut file: fs::File =
                            fs::File::create(&new_file_name).await.to_lua_err()?;
                        file.write_all(&this.content).await.to_lua_err()?;
                        let f_name: &str = new_file_name.to_str().unwrap_or("");
                        return Ok((true, f_name.to_string()));
                    } else {
                        return Ok((false, "".to_string()));
                    }
                } else if path_len >= 2 {
                    if let LuaValue::String(v) = &path[0] {
                        let mut p: String = v.to_str()?.to_string();
                        let last_char: char = p.pop().unwrap_or('/');
                        if last_char == '/' {
                            p.push('/');
                        } else {
                            p.push(last_char);
                            p.push('/');
                        }

                        let dir: &Path = Path::new(&p);

                        if !dir.exists() {
                            fs::create_dir_all(dir).await.to_lua_err()?;
                        }
                        if let LuaValue::String(f) = &path[1] {
                            let file_name: &str = f.to_str()?;

                            let new_file_name: std::path::PathBuf = dir.join(file_name);
                            let mut file: fs::File =
                                fs::File::create(&new_file_name).await.to_lua_err()?;
                            file.write_all(&this.content).await.to_lua_err()?;

                            let f_name: &str = new_file_name.to_str().unwrap_or("");
                            return Ok((true, f_name.to_string()));
                        } else {
                            return Ok((false, "".to_string()));
                        }
                    } else {
                        return Ok((false, "".to_string()));
                    }
                }
                Ok((false, "".to_string()))
            },
        );
        _methods.add_async_function("new", |_, file_path: String| async move {
            let path = Path::new(&file_path);

            if path.exists() {
                if path.is_file() {
                    let file_name: &str = path
                        .file_name()
                        .unwrap_or_else(|| OsStr::new("default.txt"))
                        .to_str()
                        .unwrap_or("default.txt");
                    let field_name: Vec<&str> = file_name.split('.').collect();
                    let ext: &str = path
                        .extension()
                        .unwrap_or_else(|| OsStr::new("txt"))
                        .to_str()
                        .unwrap_or("txt");

                    let mut f: fs::File = fs::File::open(path).await.to_lua_err()?;
                    let mut buffer: Vec<u8> = Vec::new();
                    f.read_to_end(&mut buffer).await?;
                    let file: FileData = FileData::new(field_name[0], file_name, ext, buffer);
                    Ok(file)
                } else {
                    Err(LuaError::ExternalError(Arc::new(WebError::new(
                        4031,
                        format!("{file_path} must be a file"),
                    ))))
                }
            } else {
                Err(LuaError::ExternalError(Arc::new(WebError::new(
                    4030,
                    format!("{file_path} Not Found"),
                ))))
            }
        });
        _methods.add_async_function("get_file", |_, this: LuaAnyUserData| async move {
            let this: FileData = this.take::<Self>()?;
            let mut builder = Response::builder().status(200);
            if this.content_type == "txt" {
                builder = builder.header("Content-Type", "text/plain");
            } else if this.content_type == "html" {
                builder = builder.header("Content-Type", "text/html");
            } else if this.content_type == "xml" {
                builder = builder.header("Content-Type", "application/xml");
            } else if this.content_type == "gif" {
                builder = builder.header("Content-Type", "image/gif");
            } else if this.content_type == "jpeg" || this.content_type == "jpg" {
                builder = builder.header("Content-Type", "image/jpeg");
            } else if this.content_type == "png" {
                builder = builder.header("Content-Type", "image/png");
            } else if this.content_type == "xhtml" {
                builder = builder.header("Content-Type", "application/xhtml+xml");
            } else if this.content_type == "json" {
                builder = builder.header("Content-Type", "application/json");
            } else if this.content_type == "pdf" {
                builder = builder.header("Content-Type", "application/pdf");
            } else if this.content_type == "docx" {
                builder = builder.header("Content-Type", "application/msword");
            } else {
                builder = builder.header("Content-Type", "application/octet-stream");
            }
            let resp = builder.body(Body::from(this.content)).to_lua_err()?;

            Ok(HiveResponse(resp))
        });
        _methods.add_async_function("download", |_, this: LuaAnyUserData| async move {
            let this: FileData = this.take::<Self>()?;
            let data = Response::builder()
                .status(200)
                .header("Content-Type", "application/octet-stream")
                .header(
                    "Content-Disposition",
                    format!("attachment;filename={}", this.file_name),
                )
                .body(Body::from(this.content))
                .to_lua_err()?;
            Ok(HiveResponse(data))
        });
    }
}
