use mlua::prelude::*;
use nanoid::nanoid;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct File {
    field_name: String,
    file_name: String,
    content_type: String,
    content: Vec<u8>,
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
        _fields.add_field_method_get("fieldName", |lua, this| {
            let field_name = lua.create_string(&this.field_name)?;
            Ok(field_name)
        });
        _fields.add_field_method_get("fileName", |lua, this| {
            let file_name = lua.create_string(&this.file_name)?;
            Ok(file_name)
        });
        _fields.add_field_method_get("contentType", |lua, this| {
            let content_type = lua.create_string(&this.content_type)?;
            Ok(content_type)
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

                            let file: Vec<&str> = this.file_name.split('.').collect();
                            let suffix = file[1];

                            let random_name = nanoid!(16, &alphabet) + ".";

                            let new_file_name = p + &random_name + suffix;
                            let mut file =
                                fs::File::create(new_file_name.clone()).await.to_lua_err()?;
                            file.write_all(&this.content).await.to_lua_err()?;
                            // fs::write(new_file_name, this.content.as_ref())
                            //     .await
                            //     .to_lua_err()?;
                            let file_name = lua.create_string(&new_file_name)?;
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

                            match path[1].clone() {
                                LuaValue::String(f) => {
                                    let file_name = f.to_str()?;

                                    let new_file_name = p + file_name;
                                    let mut file = fs::File::create(new_file_name.clone())
                                        .await
                                        .to_lua_err()?;
                                    file.write_all(&this.content).await.to_lua_err()?;
                                    // fs::write(new_file_name, this.content.as_ref())
                                    //     .await
                                    //     .to_lua_err()?;
                                    let file_name = lua.create_string(&new_file_name)?;
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
    }
}
