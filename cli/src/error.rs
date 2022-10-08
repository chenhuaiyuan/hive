use std::fmt;

use axum_core::extract::rejection::BytesRejection;
use axum_core::response::IntoResponse;
use hyper::Error as HyperError;
use mlua::Error as MLuaError;

#[derive(Debug)]
pub struct Error {
    code: u16,
    message: String,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn new<T>(code: u16, message: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            code,
            message: message.into(),
        }
    }

    pub(crate) fn parse_params(error: serde_urlencoded::de::Error) -> Self {
        Self {
            code: 3000u16,
            message: error.to_string(),
        }
    }

    pub(crate) fn invalid_form_content_type() -> Self {
        Self {
            code: 3001u16,
            message: "Invalid Form Content Type".to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code: {}, message: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> axum_core::response::Response {
        let resp = format!(
            r#"{{"code": "{}", "message": "{}"}}"#,
            self.code, self.message
        );
        resp.into_response()
    }
}

impl From<MLuaError> for Error {
    fn from(value: MLuaError) -> Self {
        Self::new(2000, value.to_string())
    }
}

impl From<BytesRejection> for Error {
    fn from(value: BytesRejection) -> Self {
        Self::new(2001, value.to_string())
    }
}

impl From<HyperError> for Error {
    fn from(value: HyperError) -> Self {
        Self::new(2002, value.to_string())
    }
}
