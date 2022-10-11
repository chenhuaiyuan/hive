use mlua::prelude::*;
use nanoid::nanoid;

pub fn create_nanoid<'a>(lua: &'a Lua) -> LuaResult<LuaFunction> {
    let func = lua.create_function(|lua, mut params: LuaMultiValue| {
        let nid;
        if params.is_empty() {
            nid = nanoid!();
        } else if params.len() == 1 {
            let idx = params.pop_front();
            if let Some(idx) = idx {
                match idx {
                    LuaValue::Integer(i) => {
                        let i = i as usize;
                        nid = nanoid!(i);
                    }
                    _ => nid = nanoid!(),
                }
            } else {
                nid = nanoid!();
            }
        } else {
            let idx = params.pop_front();
            let alphabet = params.pop_front();
            if let Some(alphabet) = alphabet {
                match alphabet {
                    LuaValue::Table(v) => {
                        let mut alphabet_data = Vec::new();
                        for pair in v.pairs::<LuaValue, LuaValue>() {
                            let (_, value) = pair?;
                            match value {
                                LuaValue::Integer(v) => {
                                    let character = char::from_u32(v as u32);
                                    if let Some(character) = character {
                                        alphabet_data.push(character);
                                    }
                                }
                                LuaValue::String(v) => {
                                    let val = v.to_str()?;
                                    let mut character: Vec<char> = val.chars().collect();
                                    alphabet_data.append(&mut character);
                                }
                                _ => {}
                            }
                        }
                        if let Some(idx) = idx {
                            match idx {
                                LuaValue::Integer(i) => {
                                    let i = i as usize;
                                    nid = nanoid!(i, &alphabet_data);
                                }
                                _ => {
                                    nid = nanoid!();
                                }
                            }
                        } else {
                            nid = nanoid!();
                        }
                    }
                    _ => {
                        if let Some(idx) = idx {
                            match idx {
                                LuaValue::Integer(i) => {
                                    let i = i as usize;
                                    nid = nanoid!(i);
                                }
                                _ => {
                                    nid = nanoid!();
                                }
                            }
                        } else {
                            nid = nanoid!();
                        }
                    }
                }
            } else {
                if let Some(idx) = idx {
                    match idx {
                        LuaValue::Integer(i) => {
                            let i = i as usize;
                            nid = nanoid!(i);
                        }
                        _ => {
                            nid = nanoid!();
                        }
                    }
                } else {
                    nid = nanoid!();
                }
            }
        }
        lua.create_string(&nid)
    });
    func
}
