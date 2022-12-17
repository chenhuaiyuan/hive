use std::fmt;

#[derive(Debug)]
pub struct Error {
    code: u16,
    message: String,
}

// pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn new<T>(code: u16, message: T) -> Self
    where
        T: Into<String>,
    {
        let message = message.into();
        Self { code, message }
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
