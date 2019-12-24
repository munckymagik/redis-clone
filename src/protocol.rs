//! Implementation of the RESP protool https://redis.io/topics/protocol

use std::cmp::PartialEq;
use std::convert::TryFrom;

mod builder;
mod decode;
mod errors;
mod io;

pub use builder::RespBuilder;
pub use decode::decode;
pub use errors::{RespError, RespResult};

const DEPTH_LIMIT: usize = 512;
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

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
enum RespSym {
    SimpleString = b'+',
    Error = b'-',
    Integer = b':',
    BulkString = b'$',
    Array = b'*',
}

impl TryFrom<u8> for RespSym {
    type Error = RespError;

    fn try_from(value: u8) -> RespResult<Self> {
        match value {
            v if v == Self::SimpleString as u8 => Ok(Self::SimpleString),
            v if v == Self::Error as u8 => Ok(Self::Error),
            v if v == Self::Integer as u8 => Ok(Self::Integer),
            v if v == Self::BulkString as u8 => Ok(Self::BulkString),
            v if v == Self::Array as u8 => Ok(Self::Array),
            _ => Err(RespError::UnsupportedSymbol),
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
            RespError::UnsupportedSymbol
        );
    }
}
