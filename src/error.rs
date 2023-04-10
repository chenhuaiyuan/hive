use std::fmt;

#[cfg(feature = "create_object")]
use downloader::Error as DownloaderError;
#[cfg(feature = "hive_log")]
use fast_log::error::LogError;
use hyper::{Body, Error as HyperError};

use http::{Error as HttpError, Response};
#[cfg(any(
    feature = "lua51",
    feature = "lua52",
    feature = "lua53",
    feature = "lua54",
    feature = "luau",
    feature = "luajit",
    feature = "luajit52"
))]
use mlua::prelude::{Lua, LuaError as MLuaError, LuaFunction, LuaResult};
use multer::Error as MulterError;
#[cfg(feature = "lua_hotfix")]
use notify::Error as NotifyError;
use serde_json::Error as JsonError;
use std::io::Error as IoError;
use std::net::AddrParseError;
use std::path::StripPrefixError;
use std::string::FromUtf8Error;
#[cfg(feature = "js")]
use v8::DataError as V8DataError;
#[cfg(feature = "create_object")]
use zip::result::ZipError;

#[derive(Debug)]
pub struct Error {
    pub code: u16,
    pub message: String,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(any(
    feature = "lua51",
    feature = "lua52",
    feature = "lua53",
    feature = "lua54",
    feature = "luau",
    feature = "luajit",
    feature = "luajit52"
))]
pub fn create_error(lua: &Lua) -> LuaResult<LuaFunction> {
    use std::sync::Arc;
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

    #[allow(dead_code)]
    pub fn to_response(&self, status: u16) -> Result<Response<Body>> {
        let body = format!(
            r#"{{"code": "{}", "message": "{}"}}"#,
            self.code, self.message
        );
        let response = Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::from(body))?;
        Ok(response)
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

#[cfg(any(
    feature = "lua51",
    feature = "lua52",
    feature = "lua53",
    feature = "lua54",
    feature = "luau",
    feature = "luajit",
    feature = "luajit52"
))]
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

#[cfg(feature = "hive_log")]
impl From<LogError> for Error {
    fn from(value: LogError) -> Self {
        Self::new(2005, value.to_string())
    }
}

#[cfg(feature = "lua_hotfix")]
impl From<NotifyError> for Error {
    fn from(value: NotifyError) -> Self {
        Self::new(2006, value.to_string())
    }
}

#[cfg(feature = "create_object")]
impl From<DownloaderError> for Error {
    fn from(value: DownloaderError) -> Self {
        Self::new(2007, value.to_string())
    }
}

#[cfg(feature = "create_object")]
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

impl From<JsonError> for Error {
    fn from(value: JsonError) -> Self {
        Self::new(2008, value.to_string())
    }
}
