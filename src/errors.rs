use crate::protocol::ProtoError;
use std::error::Error as StdError;
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    EmptyRequest,
    MissingArguments(String),
    UnimplementedCommand,
    UnsupportedRequestType,
    ProtocolError,
    Proto(ProtoError),
    Io(std::io::Error),
}

impl StdError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EmptyRequest => write!(f, "Empty query"),
            Self::MissingArguments(ref cmd) => {
                write!(f, "wrong number of arguments for '{}' command", cmd)
            }
            Self::UnimplementedCommand => write!(f, "Unimplemented command"),
            Self::UnsupportedRequestType => write!(f, "Unsupported request type"),
            Self::ProtocolError => write!(f, "Protocol error: expected '$', got something else"),
            Self::Proto(ref source) => write!(f, "{}", source),
            Self::Io(ref source) => write!(f, "{}", source),
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::EmptyRequest, &Self::EmptyRequest) => true,
            (&Self::MissingArguments(ref a), &Self::MissingArguments(ref b)) => a == b,
            (&Self::UnimplementedCommand, &Self::UnimplementedCommand) => true,
            (&Self::UnsupportedRequestType, &Self::UnsupportedRequestType) => true,
            (&Self::ProtocolError, &Self::ProtocolError) => true,
            (&Self::Proto(ref a), &Self::Proto(ref b)) => a == b,
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
