use mlua::prelude::*;
use std::time::Duration;

pub struct TimeDuration(pub Duration);

pub fn create_duration(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    lua.create_proxy::<TimeDuration>()
}

impl LuaUserData for TimeDuration {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("new", |_, (secs, nanos): (u64, u32)| {
            Ok(TimeDuration(Duration::new(secs, nanos)))
        });
        _methods.add_function("from_secs", |_, secs: u64| {
            Ok(TimeDuration(Duration::from_secs(secs)))
        });
        _methods.add_function("from_millis", |_, millis: u64| {
            Ok(TimeDuration(Duration::from_millis(millis)))
        });
        _methods.add_function("from_micros", |_, micros: u64| {
            Ok(TimeDuration(Duration::from_micros(micros)))
        });
        _methods.add_function("from_nanos", |_, nanos: u64| {
            Ok(TimeDuration(Duration::from_nanos(nanos)))
        });
        _methods.add_method("is_zero", |_, this, ()| Ok(this.0.is_zero()));
        _methods.add_method("as_secs", |_, this, ()| Ok(this.0.as_secs()));
        _methods.add_method("subsec_millis", |_, this, ()| Ok(this.0.subsec_millis()));
        _methods.add_method("subsec_micros", |_, this, ()| Ok(this.0.subsec_micros()));
        _methods.add_method("subsec_nanos", |_, this, ()| Ok(this.0.subsec_nanos()));
        _methods.add_method("as_millis", |_, this, ()| Ok(this.0.as_millis()));
        _methods.add_method("as_micros", |_, this, ()| Ok(this.0.as_micros()));
        _methods.add_method("as_nanos", |_, this, ()| Ok(this.0.as_nanos()));
        _methods.add_function(
            "checked_add",
            |lua, (this, rhs): (LuaAnyUserData, LuaAnyUserData)| {
                let this = this.take::<Self>()?;
                let duration = rhs.take::<TimeDuration>()?;
                let new_duration = this.0.checked_add(duration.0);
                if let Some(nd) = new_duration {
                    Ok(LuaValue::UserData(lua.create_userdata(TimeDuration(nd))?))
                } else {
                    Ok(LuaValue::Nil)
                }
            },
        );
        _methods.add_meta_function(
            LuaMetaMethod::Add,
            |_, (lhs, rhs): (LuaAnyUserData, LuaAnyUserData)| {
                let this = lhs.take::<Self>()?;
                let rhs = rhs.take::<TimeDuration>()?;
                Ok(TimeDuration(this.0.saturating_add(rhs.0)))
            },
        );
        // TODO
    }
}
