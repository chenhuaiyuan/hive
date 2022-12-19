use base64::{decode, encode};
use mlua::prelude::*;
use sha2::{Digest, Sha256};
pub struct LuaCrypto;

impl LuaUserData for LuaCrypto {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("password_hash", |lua, password: String| {
            let alphabet = [
                '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
            ];
            let salt = nanoid::nanoid!(7, &alphabet);
            let mut hasher = Sha256::new();
            hasher.update(password.as_bytes());
            let data = format!("{:x}${}", hasher.finalize(), salt);
            let pass = lua.create_string(&encode(data))?;
            Ok(pass)
        });
        _methods.add_function(
            "verify_password",
            |_, (password_hash, password): (String, String)| {
                let old_pass = decode(password_hash).to_lua_err()?;
                let old_pass = String::from_utf8(old_pass).to_lua_err()?;
                let pass: Vec<&str> = old_pass.split('$').collect();
                let mut hasher = Sha256::new();
                hasher.update(password.as_bytes());
                let new_pass = format!("{:x}", hasher.finalize());
                if pass[0] == new_pass {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
        );
    }
}

#[mlua::lua_module]
fn crypto(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    lua.create_proxy::<LuaCrypto>()
}
