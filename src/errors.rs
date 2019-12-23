use std::error::Error as StdError;
use std::fmt::{self, Display};

pub type BoxedError = Box<dyn StdError>;
pub type Result<T> = std::result::Result<T, ProtoError>;

#[derive(Debug)]
pub enum ProtoError {
    ConnectionClosed,
    ExceededDepthLimit,
    ExceededMaxLineLength,
    InvalidArraySize,
    InvalidBulkStringSize,
    InvalidTerminator,
    UnsupportedSymbol,
    Message(String),
    BoxedError(BoxedError),
}

impl StdError for ProtoError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::BoxedError(e) => Some(&**e),
            _ => None,
        }
    }
}

impl Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConnectionClosed => write!(f, "ConnectionClosed"),
            Self::ExceededDepthLimit => write!(f, "ExceededDepthLimit"),
            Self::ExceededMaxLineLength => write!(f, "ExceededMaxLineLength"),
            Self::InvalidArraySize => write!(f, "InvalidArraySize"),
            Self::InvalidBulkStringSize => write!(f, "InvalidBulkStringSize"),
            Self::InvalidTerminator => write!(f, "InvalidTerminator"),
            Self::UnsupportedSymbol => write!(f, "UnsupportedSymbol"),
            Self::Message(msg) => write!(f, "{}", msg),
            Self::BoxedError(other) => write!(f, "{}", other),
        }
    }
}

impl PartialEq for ProtoError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::ConnectionClosed, &Self::ConnectionClosed) => true,
            (&Self::ExceededDepthLimit, &Self::ExceededDepthLimit) => true,
            (&Self::ExceededMaxLineLength, &Self::ExceededMaxLineLength) => true,
            (&Self::InvalidArraySize, &Self::InvalidArraySize) => true,
            (&Self::InvalidBulkStringSize, &Self::InvalidBulkStringSize) => true,
            (&Self::InvalidTerminator, &Self::InvalidTerminator) => true,
            (&Self::UnsupportedSymbol, &Self::UnsupportedSymbol) => true,
            (&Self::Message(ref a), &Self::Message(ref b)) => a == b,
            _ => false,
        }
    }
}

impl From<&str> for ProtoError {
    fn from(other: &str) -> Self {
        Self::Message(other.to_owned())
    }
}

impl From<String> for ProtoError {
    fn from(other: String) -> Self {
        Self::Message(other)
    }
}

impl From<std::str::Utf8Error> for ProtoError {
    fn from(other: std::str::Utf8Error) -> Self {
        Self::BoxedError(other.into())
    }
}

impl From<std::io::Error> for ProtoError {
    fn from(other: std::io::Error) -> Self {
        Self::BoxedError(other.into())
    }
}

impl From<std::num::ParseIntError> for ProtoError {
    fn from(other: std::num::ParseIntError) -> Self {
        Self::BoxedError(other.into())
    }
}

impl From<std::num::TryFromIntError> for ProtoError {
    fn from(other: std::num::TryFromIntError) -> Self {
        Self::BoxedError(other.into())
    }
}
