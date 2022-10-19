use std::collections::HashMap;

use jwt_simple::{
    algorithms::HS256Key,
    claims::Claims,
    prelude::{Duration, MACLike, VerificationOptions},
};
use mlua::prelude::*;
use serde_json::Value as JsonValue;

pub struct HS256 {
    key: HS256Key,
    duration: Option<Duration>,
}

impl LuaUserData for HS256 {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("new", |_, key: String| {
            let hs256 = HS256Key::from_bytes(key.as_bytes());
            Ok(Self {
                key: hs256,
                duration: None,
            })
        });
        _methods.add_method_mut("setDays", |_, this, days: u64| {
            this.duration = Some(Duration::from_days(days));
            Ok(())
        });
        _methods.add_method_mut("setHours", |_, this, hours: u64| {
            this.duration = Some(Duration::from_hours(hours));
            Ok(())
        });
        _methods.add_method("generateToken", |_, this, data: LuaTable| {
            let claims;
            let mut token_data = HashMap::new();
            for pair in data.pairs::<LuaValue, LuaValue>() {
                let (key, value) = pair?;
                if let LuaValue::String(k) = key {
                    let k = k.to_str()?.to_string();
                    match value {
                        LuaValue::Integer(v) => {
                            token_data.insert(k, JsonValue::from(v));
                        }
                        LuaValue::Number(v) => {
                            token_data.insert(k, JsonValue::from(v));
                        }
                        LuaValue::String(v) => {
                            token_data.insert(k, JsonValue::from(v.to_str()?));
                        }
                        LuaValue::Nil => {
                            token_data.insert(k, JsonValue::Null);
                        }
                        LuaValue::Boolean(v) => {
                            token_data.insert(k, JsonValue::Bool(v));
                        }
                        _ => {}
                    }
                }
            }

            if let Some(duration) = this.duration {
                claims = Claims::with_custom_claims(token_data, duration);
            } else {
                claims = Claims::with_custom_claims(token_data, Duration::from_hours(1));
            }
            this.key.authenticate(claims).to_lua_err()
        });
        _methods.add_method("verify", |lua, this, token: String| {
            let options = VerificationOptions {
                accept_future: true,
                ..Default::default()
            };
            let data = this
                .key
                .verify_token::<HashMap<String, JsonValue>>(&token, Some(options));

            match data {
                Ok(val) => {
                    let custom = val.custom;
                    let res = lua.create_table()?;
                    for (key, val) in custom {
                        res.set(key, json_value_to_lua_value(val, lua)?)?;
                    }
                    Ok((LuaValue::Boolean(true), res))
                }
                Err(_) => Ok((LuaValue::Boolean(false), lua.create_table()?)),
            }
        });
    }
}

fn json_value_to_lua_value(val: JsonValue, lua: &Lua) -> LuaResult<LuaValue> {
    match val {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(v) => Ok(LuaValue::Boolean(v)),
        JsonValue::Number(v) => {
            if v.is_i64() {
                let num = v.as_i64();
                if let Some(num) = num {
                    return Ok(LuaValue::Integer(num));
                }
            } else if v.is_u64() {
                let num = v.as_u64();
                if let Some(num) = num {
                    return Ok(LuaValue::Number(num as f64));
                }
            } else if v.is_f64() {
                let num = v.as_f64();
                if let Some(num) = num {
                    return Ok(LuaValue::Number(num));
                }
            }
            return Ok(LuaValue::Integer(0));
        }
        JsonValue::String(v) => {
            let s = lua.create_string(&v)?;
            return Ok(LuaValue::String(s));
        }
        _ => Ok(LuaValue::Nil),
    }
}
