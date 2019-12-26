use crate::protocol::RespError;
use std::error::Error as StdError;
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    EmptyQuery,
    UnimplementedCommand,
    UnsupportedRequestType,
    ProtocolError,
    Resp(RespError),
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyQuery => write!(f, "Empty query"),
            Self::UnimplementedCommand => write!(f, "Unimplemented command"),
            Self::UnsupportedRequestType => write!(f, "Unsupported request type"),
            Self::ProtocolError => write!(f, "Protocol error: expected '$', got something else"),
            Self::Resp(ref source) => write!(f, "{}", source),
        }
    }
}

impl From<RespError> for Error {
    fn from(other: RespError) -> Self {
        Self::Resp(other)
    }
}
