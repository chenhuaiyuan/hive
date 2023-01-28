use mlua::prelude::*;
use rusqlite::OpenFlags;

pub struct SqliteOpenFlags(pub OpenFlags);

impl LuaUserData for SqliteOpenFlags {}

pub fn create_open_flags(lua: &Lua) -> LuaResult<LuaTable> {
    lua.create_table_from([
        (
            "SQLITE_OPEN_READ_ONLY",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_READ_ONLY),
        ),
        (
            "SQLITE_OPEN_READ_WRITE",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_READ_WRITE),
        ),
        (
            "SQLITE_OPEN_CREATE",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_CREATE),
        ),
        (
            "SQLITE_OPEN_URI",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_URI),
        ),
        (
            "SQLITE_OPEN_MEMORY",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_MEMORY),
        ),
        (
            "SQLITE_OPEN_NO_MUTEX",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_NO_MUTEX),
        ),
        (
            "SQLITE_OPEN_FULL_MUTEX",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_FULL_MUTEX),
        ),
        (
            "SQLITE_OPEN_SHARED_CACHE",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_SHARED_CACHE),
        ),
        (
            "SQLITE_OPEN_PRIVATE_CACHE",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_PRIVATE_CACHE),
        ),
        (
            "SQLITE_OPEN_NOFOLLOW",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_NOFOLLOW),
        ),
        (
            "SQLITE_OPEN_EXRESCODE",
            SqliteOpenFlags(OpenFlags::SQLITE_OPEN_EXRESCODE),
        ),
    ])
}
