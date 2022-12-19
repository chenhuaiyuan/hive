use mlua::prelude::*;
use nanoid::nanoid;

#[mlua::lua_module]
pub fn nanoid(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, mut params: LuaMultiValue| {
        let nid = if params.is_empty() {
            nanoid!()
        } else if params.len() == 1 {
            let idx = params.pop_front();
            if let Some(idx) = idx {
                match idx {
                    LuaValue::Integer(i) => {
                        let i = i as usize;
                        nanoid!(i)
                    }
                    _ => nanoid!(),
                }
            } else {
                nanoid!()
            }
        } else {
            let idx = params.pop_front();
            let alphabet = params.pop_front();
            let mut alphabet_data = Vec::new();
            if let Some(LuaValue::Table(v)) = alphabet {
                for pair in v.pairs::<LuaValue, LuaValue>() {
                    let (_, value) = pair?;
                    if let LuaValue::Integer(v) = value {
                        let character = char::from_u32(v as u32);
                        if let Some(character) = character {
                            alphabet_data.push(character);
                        }
                    } else if let LuaValue::String(v) = value {
                        let val = v.to_str()?;
                        let mut character: Vec<char> = val.chars().collect();
                        alphabet_data.append(&mut character);
                    }
                }
            }

            if let Some(idx) = idx {
                match idx {
                    LuaValue::Integer(i) => {
                        let i = i as usize;
                        if alphabet_data.is_empty() {
                            nanoid!(i)
                        } else {
                            nanoid!(i, &alphabet_data)
                        }
                    }
                    _ => nanoid!(),
                }
            } else {
                nanoid!()
            }
        };
        lua.create_string(&nid)
    })
}
