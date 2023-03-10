use std::fmt;
#[cfg(feature = "lua")]
use std::sync::Arc;

use downloader::Error as DownloaderError;
use fast_log::error::LogError;
use hyper::Error as HyperError;

use http::Error as HttpError;
#[cfg(feature = "lua")]
use mlua::prelude::{Lua, LuaError as MLuaError, LuaFunction, LuaResult};
use multer::Error as MulterError;
use notify::Error as NotifyError;
use std::io::Error as IoError;
use std::net::AddrParseError;
use std::path::StripPrefixError;
use std::string::FromUtf8Error;
#[cfg(feature = "js")]
use v8::DataError as V8DataError;
use zip::result::ZipError;

#[derive(Debug)]
pub struct Error {
    code: u16,
    message: String,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(feature = "lua")]
pub fn create_error(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, (code, message): (u16, String)| {
        log::error!("{message}");
        let err: Error = Error::new(code, message);
        Err::<(), MLuaError>(MLuaError::ExternalError(Arc::new(err)))
    })
}

impl Error {
    pub fn new<T>(code: u16, message: T) -> Self
    where
        T: Into<String>,
    {
        let message: String = message.into();
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

#[cfg(feature = "lua")]
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

impl From<DownloaderError> for Error {
    fn from(value: DownloaderError) -> Self {
        Self::new(2007, value.to_string())
    }
}

impl From<ZipError> for Error {
    fn from(value: ZipError) -> Self {
        Self::new(2007, value.to_string())
    }
}

impl From<IoError> for Error {
    fn from(value: IoError) -> Self {
        Self::new(2007, value.to_string())
    }
}

impl From<StripPrefixError> for Error {
    fn from(value: StripPrefixError) -> Self {
        Self::new(2007, value.to_string())
    }
}

impl From<MulterError> for Error {
    fn from(value: MulterError) -> Self {
        Self::new(2007, value.to_string())
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Self::new(2007, value.to_string())
    }
}

impl From<HttpError> for Error {
    fn from(value: HttpError) -> Self {
        Self::new(2007, value.to_string())
    }
}

#[cfg(feature = "js")]
impl From<V8DataError> for Error {
    fn from(value: V8DataError) -> Self {
        Self::new(2007, value.to_string())
    }
}
