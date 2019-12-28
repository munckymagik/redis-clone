//! Implementation of the RESP protool https://redis.io/topics/protocol

use std::cmp::PartialEq;
use std::convert::TryFrom;

mod decode;
mod errors;

pub use decode::decode;
pub use errors::{RespError, RespResult};

const DEPTH_LIMIT: usize = 420; // TODO reduced from 512 to avoid stack overflow in test
const MAX_ARRAY_SIZE: usize = 1024 * 1024;
const MAX_BULK_STR_SIZE: usize = 512 * 1024 * 1024;
const MAX_LINE_LENGTH: usize = 64 * 1024;
const LF: u8 = b'\n';
const CRLF: &[u8] = b"\r\n";

#[derive(Debug, PartialEq, Eq)]
pub enum RespVal {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Option<Vec<RespVal>>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RespSym {
    SimpleString = b'+',
    Error = b'-',
    Integer = b':',
    BulkString = b'$',
    Array = b'*',
}

impl RespSym {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn as_char(&self) -> char {
        char::from(self.as_u8())
    }
}

impl TryFrom<u8> for RespSym {
    type Error = RespError;

    fn try_from(value: u8) -> RespResult<Self> {
        use RespSym::*;

        match value {
            v if v == SimpleString.as_u8() => Ok(SimpleString),
            v if v == Error.as_u8() => Ok(Error),
            v if v == Integer.as_u8() => Ok(Integer),
            v if v == BulkString.as_u8() => Ok(BulkString),
            v if v == Array.as_u8() => Ok(Array),
            v => Err(RespError::UnsupportedSymbol(v.into())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rsym() {
        assert_eq!(RespSym::try_from(b'+').unwrap(), RespSym::SimpleString);
        assert_eq!(RespSym::try_from(b'-').unwrap(), RespSym::Error);
        assert_eq!(RespSym::try_from(b':').unwrap(), RespSym::Integer);
        assert_eq!(RespSym::try_from(b'$').unwrap(), RespSym::BulkString);
        assert_eq!(RespSym::try_from(b'*').unwrap(), RespSym::Array);
        assert_eq!(
            RespSym::try_from(b'0').unwrap_err(),
            RespError::UnsupportedSymbol('0')
        );

        assert_eq!(RespSym::Array.as_u8(), b'*');
        assert_eq!(RespSym::Array.as_char(), '*');
    }
}
