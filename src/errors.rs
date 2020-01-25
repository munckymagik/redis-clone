use crate::protocol::ProtoError;
use std::error::Error as StdError;
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    EmptyRequest,
    UnimplementedCommand,
    UnsupportedRequestType,
    Message(String),
    ProtocolError,
    Proto(ProtoError),
    Io(std::io::Error),
    CastingInt(std::num::TryFromIntError),
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyRequest => write!(f, "Empty query"),
            Self::UnimplementedCommand => write!(f, "Unimplemented command"),
            Self::UnsupportedRequestType => write!(f, "Unsupported request type"),
            Self::Message(ref msg) => write!(f, "{}", msg),
            Self::ProtocolError => write!(f, "Protocol error: expected '$', got something else"),
            Self::Proto(ref source) => write!(f, "{}", source),
            Self::Io(ref source) => write!(f, "{}", source),
            Self::CastingInt(ref source) => write!(f, "{}", source),
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::EmptyRequest, &Self::EmptyRequest) => true,
            (&Self::UnimplementedCommand, &Self::UnimplementedCommand) => true,
            (&Self::UnsupportedRequestType, &Self::UnsupportedRequestType) => true,
            (&Self::Message(ref a), &Self::Message(ref b)) => a == b,
            (&Self::ProtocolError, &Self::ProtocolError) => true,
            (&Self::Proto(ref a), &Self::Proto(ref b)) => a == b,
            (&Self::CastingInt(ref a), &Self::CastingInt(ref b)) => a == b,
            (&Self::Io(_), &Self::Io(_)) => false, // cannot be compared
            _ => false,
        }
    }
}

impl From<ProtoError> for Error {
    fn from(other: ProtoError) -> Self {
        match other {
            ProtoError::EmptyRequest => Self::EmptyRequest,
            _ => Self::Proto(other),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Self::Io(other)
    }
}

impl From<&str> for Error {
    fn from(other: &str) -> Self {
        Self::from(other.to_owned())
    }
}

impl From<String> for Error {
    fn from(other: String) -> Self {
        Self::Message(other)
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(other: std::num::TryFromIntError) -> Self {
        Self::CastingInt(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_resp_error() {
        assert_eq!(
            Error::from(ProtoError::InvalidTerminator),
            Error::Proto(ProtoError::InvalidTerminator)
        );

        assert_eq!(Error::from(ProtoError::EmptyRequest), Error::EmptyRequest,);
    }
}
