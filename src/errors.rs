use std::{error, fmt};

pub type BoxedError = Box<dyn error::Error>;
pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug)]
pub enum ServerError {
    EmptyRead,
    Message(String),
    BoxedError(BoxedError),
}

impl error::Error for ServerError {}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyRead => write!(f, "EmptyRead"),
            Self::Message(msg) => write!(f, "{}", msg),
            Self::BoxedError(other) => write!(f, "{}", other),
        }
    }
}

impl From<&str> for ServerError {
    fn from(other: &str) -> Self {
        Self::Message(other.to_owned())
    }
}

impl From<String> for ServerError {
    fn from(other: String) -> Self {
        Self::Message(other)
    }
}

impl From<std::str::Utf8Error> for ServerError {
    fn from(other: std::str::Utf8Error) -> Self {
        Self::BoxedError(other.into())
    }
}

impl From<std::io::Error> for ServerError {
    fn from(other: std::io::Error) -> Self {
        Self::BoxedError(other.into())
    }
}
