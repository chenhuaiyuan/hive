use std::fmt;
use std::sync::Arc;

use fast_log::error::LogError;
use hyper::Error as HyperError;
use mlua::prelude::{Lua, LuaError as MLuaError, LuaFunction, LuaResult};
use notify::Error as NotifyError;
use std::net::AddrParseError;

#[derive(Debug)]
pub struct Error {
    code: u16,
    message: String,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn create_error(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, (code, message): (u16, String)| {
        let err = Error::new(code, message.clone());
        log::error!("{}", message);
        Err::<(), MLuaError>(MLuaError::ExternalError(Arc::new(err)))
    })
}

impl Error {
    pub fn new<T>(code: u16, message: T) -> Self
    where
        T: Into<String>,
    {
        let message = message.into();
        log::error!("{}", message);
        Self { code, message }
    }

    pub(crate) fn parse_params(error: serde_urlencoded::de::Error) -> Self {
        log::error!("{}", error.to_string());
        Self {
            code: 3000u16,
            message: error.to_string(),
        }
    }

    // pub(crate) fn invalid_form_content_type() -> Self {
    //     Self {
    //         code: 3001u16,
    //         message: "Invalid Form Content Type".to_string(),
    //     }
    // }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl From<MLuaError> for Error {
    fn from(value: MLuaError) -> Self {
        Self::new(2000, value.to_string())
    }
}

impl From<HyperError> for Error {
    fn from(value: HyperError) -> Self {
        Self::new(2002, value.to_string())
    }
}

impl From<AddrParseError> for Error {
    fn from(value: AddrParseError) -> Self {
        Self::new(2003, value.to_string())
    }
}

impl From<LogError> for Error {
    fn from(value: LogError) -> Self {
        Self::new(2005, value.to_string())
    }
}

impl From<NotifyError> for Error {
    fn from(value: NotifyError) -> Self {
        Self::new(2006, value.to_string())
    }
}
