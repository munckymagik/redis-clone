/// Implementation of the RESP protool https://redis.io/topics/protocol
use std::cmp::PartialEq;
use std::convert::{TryFrom, TryInto};
use std::io::{BufRead, Read};

use crate::errors::{RespError, Result};

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

    fn try_from(value: u8) -> Result<Self> {
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

pub fn decode<T: BufRead>(mut stream: T) -> Result<RespVal> {
    do_decode(&mut stream, 0)
}

fn do_decode(stream: &mut impl BufRead, depth: usize) -> Result<RespVal> {
    if depth > DEPTH_LIMIT {
        return Err(RespError::ExceededDepthLimit);
    }

    let (type_sym, value_str) = read_header(stream)?;

    let value = match type_sym {
        RespSym::SimpleString => RespVal::SimpleString(value_str.into()),
        RespSym::Error => RespVal::Error(value_str.into()),
        RespSym::Integer => RespVal::Integer(value_str.parse()?),
        RespSym::BulkString => {
            let len = value_str
                .parse()
                .or(Err(RespError::InvalidBulkStringSize))?;
            let value = read_bulk_string(stream, len)?;
            RespVal::BulkString(value)
        }
        RespSym::Array => {
            let len = value_str.parse().or(Err(RespError::InvalidArraySize))?;
            let value = read_array(stream, len, depth)?;
            RespVal::Array(value)
        }
    };

    Ok(value)
}

fn read_header(stream: &mut impl BufRead) -> Result<(RespSym, String)> {
    let mut buffer = vec![];
    read_line(stream, &mut buffer)?;

    let (&type_sym, tail) = buffer
        .split_first()
        .ok_or_else(|| RespError::from("Error parsing resp header structure"))?;

    let type_sym = RespSym::try_from(type_sym)?;
    let tail = std::str::from_utf8(tail)?.to_owned();

    Ok((type_sym, tail))
}

fn read_line(stream: &mut impl BufRead, buffer: &mut Vec<u8>) -> Result<()> {
    let limit = MAX_LINE_LENGTH.try_into().unwrap();
    let num_bytes = stream.take(limit).read_until(LF, buffer)?;

    // If we got nothing then we can assume the connection has closed
    if num_bytes == 0 {
        return Err(RespError::ConnectionClosed);
    }
    // We must have at least 2 bytes for CRLF
    if num_bytes < 2 {
        return Err(RespError::InvalidTerminator);
    }
    // The line must be terminated by CRLF
    if &buffer[(num_bytes - 2)..] != CRLF {
        // We may be missing the CRLF because the line limit has been exceeded
        if num_bytes == MAX_LINE_LENGTH {
            return Err(RespError::ExceededMaxLineLength);
        }

        return Err(RespError::InvalidTerminator);
    }

    // Drop the CRLF
    buffer.truncate(num_bytes - 2);

    Ok(())
}

fn read_bulk_string(stream: &mut impl BufRead, len: i64) -> Result<Option<String>> {
    if len == -1 {
        return Ok(None);
    }

    let len = usize::try_from(len).or(Err(RespError::InvalidBulkStringSize))?;

    if len > MAX_BULK_STR_SIZE {
        return Err(RespError::InvalidBulkStringSize);
    }

    let mut buffer = vec![0; len + 2];
    stream.read_exact(&mut buffer)?;
    let value_str = std::str::from_utf8(&buffer[..len])?;

    Ok(Some(value_str.to_owned()))
}

fn read_array(stream: &mut impl BufRead, len: i64, depth: usize) -> Result<Option<Vec<RespVal>>> {
    if len == -1 {
        return Ok(None);
    }

    let len = usize::try_from(len).or(Err(RespError::InvalidArraySize))?;

    if len > MAX_ARRAY_SIZE {
        return Err(RespError::InvalidArraySize);
    }

    let mut elements = Vec::with_capacity(len);

    for _ in 0..len {
        let elem = do_decode(stream, depth + 1)?;
        elements.push(elem);
    }

    Ok(Some(elements))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_line() {
        use std::io::Cursor;
        use std::io::Write;

        let mut stream = Cursor::new(Vec::new());
        stream.write_all(b"123\r\n456\r\n").unwrap();
        stream.set_position(0);
        let mut buffer = vec![];

        read_line(&mut stream, &mut buffer).unwrap();
        assert_eq!(buffer, b"123");

        buffer.clear();
        read_line(&mut stream, &mut buffer).unwrap();
        assert_eq!(buffer, b"456");

        buffer.clear();
        assert_eq!(
            read_line(&mut stream, &mut buffer).unwrap_err(),
            RespError::ConnectionClosed
        );
    }

    #[test]
    fn test_read_line_invalid_terminator() {
        // Valid case: an empty line
        let mut buffer = vec![];
        let mut input: &[u8] = b"\r\n";
        read_line(&mut input, &mut buffer).unwrap();
        assert_eq!(buffer, b"");

        // Invalid case: a single LF without a CR
        let mut buffer = vec![];
        let mut input: &[u8] = b"\n";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.unwrap_err(), RespError::InvalidTerminator);

        // Invalid case: a single LF preceeded by something other than a CR
        let mut buffer = vec![];
        let mut input: &[u8] = b"x\n";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.unwrap_err(), RespError::InvalidTerminator);

        // Invalid case: a single CR followed by not a LF
        let mut buffer = vec![];
        let mut input: &[u8] = b"\rx";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.unwrap_err(), RespError::InvalidTerminator);

        // Invalid case: no terminator
        let mut buffer = vec![];
        let mut input: &[u8] = b"x";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.unwrap_err(), RespError::InvalidTerminator);
    }

    #[test]
    fn test_read_line_gt_max() {
        let input: Vec<u8> = b"0".repeat(MAX_LINE_LENGTH + 1);

        let result = decode(input.as_slice());
        assert_eq!(result.unwrap_err(), RespError::ExceededMaxLineLength);
    }

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

    #[test]
    fn decode_simple_string() {
        let input: &[u8] = b"+OK\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RespVal::SimpleString("OK".into()));
    }

    #[test]
    fn decode_error() {
        let input: &[u8] = b"-Error message\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RespVal::Error("Error message".into()));
    }

    #[test]
    fn decode_integer() {
        let input: &[u8] = b":1000\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RespVal::Integer(1000i64));
    }

    #[test]
    fn decode_bulk_string() {
        let input: &[u8] = b"$8\r\nabc\r\ndef\r\n";
        let result = decode(input);
        assert_eq!(
            result.unwrap(),
            RespVal::BulkString(Some("abc\r\ndef".into()))
        );
    }

    #[test]
    fn decode_empty_bulk_string() {
        let input: &[u8] = b"$0\r\n\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RespVal::BulkString(Some("".into())));
    }

    #[test]
    fn decode_null_bulk_string() {
        let input: &[u8] = b"$-1\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RespVal::BulkString(None));
    }

    #[test]
    fn decode_null_array() {
        let input: &[u8] = b"*-1\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RespVal::Array(None));
    }

    #[test]
    fn decode_empty_array() {
        let input: &[u8] = b"*0\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RespVal::Array(Some(vec![])));
    }

    #[test]
    fn decode_array_of_mixed() {
        let input: &[u8] = b"\
                                *5\r\n\
                                +1\r\n\
                                -2\r\n\
                                :3\r\n\
                                $1\r\n4\r\n\
                                *1\r\n\
                                *1\r\n\
                                *-1\r\n\
                            ";
        let result = decode(input);
        assert_eq!(
            result.unwrap(),
            RespVal::Array(Some(vec![
                RespVal::SimpleString("1".into()),
                RespVal::Error("2".into()),
                RespVal::Integer(3),
                RespVal::BulkString(Some("4".into())),
                RespVal::Array(Some(vec![RespVal::Array(Some(
                    vec![RespVal::Array(None),]
                )),]))
            ]))
        );
    }

    #[test]
    fn limit_recursion() {
        // Recurse beyond the limit
        let input: Vec<u8> = b"*1\r\n".repeat(DEPTH_LIMIT + 1);

        let result = decode(input.as_slice());
        assert_eq!(result.unwrap_err(), RespError::ExceededDepthLimit);
    }

    #[test]
    fn array_invalid_size_overflow() {
        // i64 max + 1
        let input: &[u8] = b"*9223372036854775808\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), RespError::InvalidArraySize);
    }

    #[test]
    fn array_invalid_size_negative() {
        let input: &[u8] = b"*-2\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), RespError::InvalidArraySize);
    }

    #[test]
    fn array_invalid_size_gt_max() {
        // 1024 * 1024 + 1 is too large
        let input: &[u8] = b"*1048577\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), RespError::InvalidArraySize);
    }

    #[test]
    fn bulk_string_invalid_size_overflow() {
        // i64 max + 1
        let input: &[u8] = b"$9223372036854775808\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), RespError::InvalidBulkStringSize);
    }

    #[test]
    fn bulk_string_invalid_size_negative() {
        let input: &[u8] = b"$-2\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), RespError::InvalidBulkStringSize);
    }

    #[test]
    fn bulk_string_invalid_size_gt_max() {
        // 512 * 1024 * 1024 + 1 is too large
        let input: &[u8] = b"$536870913\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), RespError::InvalidBulkStringSize);
    }
}
