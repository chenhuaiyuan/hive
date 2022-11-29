use bson::*;
use mlua::prelude::*;

pub fn create_bson(lua: &Lua) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    table.set("uuid", create_uuid(lua)?)?;
    table.set("uuid_representation", create_uuid_representation(lua)?)?;
    Ok(table)
}

pub struct BsonBinary(Binary);

impl LuaUserData for BsonBinary {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("from_uuid", |_, uuid: LuaAnyUserData| {
            let uuid = uuid.take::<BsonUuid>()?;
            Ok(BsonBinary(Binary::from(uuid.0)))
        });
        _methods.add_function(
            "from_uuid_with_representation",
            |_, (uuid, rep): (LuaAnyUserData, LuaAnyUserData)| {
                let uuid = uuid.take::<BsonUuid>()?;
                let rep = rep.take::<BsonUuidRepresentation>()?;
                Ok(BsonBinary(Binary::from_uuid_with_representation(
                    uuid.0, rep.0,
                )))
            },
        );
        _methods.add_method(
            "to_uuid_with_representation",
            |_, this, rep: LuaAnyUserData| {
                let rep = rep.take::<BsonUuidRepresentation>()?;
                let uuid = this.0.to_uuid_with_representation(rep.0).to_lua_err()?;
                Ok(BsonUuid(uuid))
            },
        );
        _methods.add_method("to_uuid", |_, this, ()| {
            let uuid = this.0.to_uuid().to_lua_err()?;
            Ok(BsonUuid(uuid))
        });
    }
}

pub struct BsonUuid(Uuid);

fn create_uuid(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| Ok(BsonUuid(Uuid::new())))
}

impl LuaUserData for BsonUuid {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("from_bytes", |_, bytes: [u8; 16]| {
            Ok(BsonUuid(Uuid::from_bytes(bytes)))
        });
        _methods.add_function("parse_str", |_, input: String| {
            let uuid = Uuid::parse_str(input).to_lua_err()?;
            Ok(BsonUuid(uuid))
        });
        _methods.add_function("bytes", |lua, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            Ok(this.0.bytes())
        });
    }
}

pub struct BsonUuidRepresentation(UuidRepresentation);

impl LuaUserData for BsonUuidRepresentation {}

fn create_uuid_representation(lua: &Lua) -> LuaResult<LuaTable> {
    lua.create_table_from([
        (
            "standard",
            lua.create_userdata(BsonUuidRepresentation(UuidRepresentation::Standard))?,
        ),
        (
            "c_sharp_legacy",
            lua.create_userdata(BsonUuidRepresentation(UuidRepresentation::CSharpLegacy))?,
        ),
        (
            "java_legacy",
            lua.create_userdata(BsonUuidRepresentation(UuidRepresentation::JavaLegacy))?,
        ),
        (
            "python_legacy",
            lua.create_userdata(BsonUuidRepresentation(UuidRepresentation::PythonLegacy))?,
        ),
    ])
}

pub struct BsonDateTime(DateTime);

// fn create_datetime(lua: &Lua) -> LuaResult<LuaAnyUserData> {}

impl LuaUserData for BsonDateTime {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {}
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {}
}
