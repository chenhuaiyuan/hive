use super::lua_is_array;
use bson::{oid::ObjectId, Binary, Bson, DateTime, Decimal128, Document, Timestamp, Uuid};
use mlua::prelude::*;

pub fn create_bson(lua: &Lua) -> LuaResult<LuaTable> {
    let table = lua.create_table()?;
    table.set("uuid", lua.create_proxy::<BsonUuid>()?)?;
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

impl LuaUserData for BsonUuid {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("new", |_, ()| Ok(BsonUuid(Uuid::new())));
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

pub struct BsonObjectId(ObjectId);

impl LuaUserData for BsonObjectId {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("new", |_, ()| Ok(BsonObjectId(ObjectId::new())));
        _methods.add_function("from_bytes", |_, bytes: [u8; 12]| {
            Ok(BsonObjectId(ObjectId::from_bytes(bytes)))
        });
        _methods.add_function("parse_str", |s: String| {
            let obj_id = ObjectId::parse_str(s).to_lua_err()?;
            Ok(BsonObjectId(obj_id))
        });
        _methods.add_method("timestamp", |_, this, ()| {
            Ok(BsonDateTime(this.0.timestamp()))
        });
        _methods.add_method("bytes", |_, this, ()| Ok(this.0.bytes()));
        _methods.add_function("to_hex", |lua, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            lua.create_string(&this.0.to_hex())
        });
    }
}

pub struct BsonDateTime(DateTime);

pub fn create_date_time(lua: &Lua) -> LuaResult<LuaAnyUserData> {
    lua.create_proxy::<BsonDateTime>()
}

impl LuaUserData for BsonDateTime {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(_fields: &mut F) {
        _fields.add_field_function_get("max", |_, _| Ok(BsonDateTime(DateTime::MAX)));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("now", |_, ()| Ok(BsonDateTime(DateTime::now())));
        // _methods.add_function("from_chrono", function)
        _methods.add_function("timestamp_millis", |lua, this: LuaAnyUserData| {
            let this = this.take::<Self>()?;
            lua.create_string(&this.0.timestamp_millis())
        });
    }
}

pub struct BsonDecimal128(Decimal128);

pub fn create_decimal128(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, bytes: [u8; 16]| Ok(BsonDecimal128(Decimal128::from_bytes(bytes))))
}

impl LuaUserData for BsonDecimal128 {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method("bytes", |_, this, ()| Ok(this.0.bytes()));
    }
}

pub struct BsonTimestamp(Timestamp);

impl LuaUserData for BsonTimestamp {}

pub fn bson_value_to_lua_value(lua: &Lua, bson_value: Bson) -> LuaResult<LuaValue> {
    match bson_value {
        Bson::Double(v) => Ok(LuaValue::Number(v)),
        Bson::String(v) => lua.create_string(&v),
        Bson::Array(val) => {
            let array = lua.create_table()?;
            let mut i = 1;
            for v in val {
                let value = bson_value_to_lua_value(&lua, v)?;
                array.set(i, value)?;
                i += 1;
            }
            Ok(LuaValue::Table(array))
        }
        Bson::Document(doc) => {
            let map = lua.create_table()?;
            for (k, v) in doc.iter() {
                map.set(k.clone(), bson_value_to_lua_value(&lua, v.clone())?)?;
            }
            Ok(LuaValue::Table(map))
        }
        Bson::Boolean(v) => Ok(LuaValue::Boolean(v)),
        Bson::Null => Ok(LuaValue::Nil),
        Bson::RegularExpression(_) => Ok(LuaValue::Nil),
        Bson::JavaScriptCode(v) => lua.create_string(&v),
        Bson::JavaScriptCodeWithScope(v) => Ok(LuaValue::Nil),
        Bson::Int32(v) => Ok(LuaValue::Integer(v as i64)),
        Bson::Int64(v) => Ok(LuaValue::Integer(v)),
        Bson::Timestamp(v) => Ok(LuaValue::Integer((v.time + v.increment) as i64)),
        Bson::Binary(v) => lua.create_userdata(BsonBinary(v)),
        Bson::ObjectId(v) => lua.create_userdata(BsonObjectId(v)),
        Bson::DateTime(v) => lua.create_userdata(BsonDateTime(v)),
        Bson::Symbol(v) => lua.create_string(&v),
        Bson::Decimal128(v) => lua.create_userdata(BsonDecimal128(v)),
        _ => Ok(LuaValue::Nil),
    }
}

pub fn lua_value_to_bson_value(lua_value: LuaValue) -> LuaResult<Bson> {
    match lua_value {
        LuaValue::Nil => Ok(Bson::Null),
        LuaValue::Boolean(v) => Ok(Bson::Boolean(v)),
        LuaValue::LightUserData(v) => Ok(Bson::Null),
        LuaValue::Integer(v) => Ok(Bson::Int64(v)),
        LuaValue::Number(v) => Ok(Bson::Double(v)),
        LuaValue::Vector(one, two, three) => Ok(Bson::Array(vec![
            Bson::Double(one as f64),
            Bson::Double(two as f64),
            Bson::Double(three as f64),
        ])),
        LuaValue::String(v) => Ok(Bson::String(v)),
        LuaValue::Table(v) => {
            let is_array = lua_is_array(v)?;
            let mut array: Vec<Bson> = Vec::new();
            if is_array {
                for pairs in v.pairs::<LuaValue, LuaValue>() {
                    let (_, val) = pairs?;
                    array.push(lua_value_to_bson_value(val)?);
                }
                Ok(Bson::Array(array))
            } else {
                let mut doc = Document::new();
                for pairs in v.pairs::<String, LuaValue>() {
                    let (key, val) = pairs?;
                    doc.insert(key, lua_value_to_bson_value(val)?)
                }
                Ok(Bson::Document(doc))
            }
        }
        _ => Ok(Bson::Null),
    }
}
