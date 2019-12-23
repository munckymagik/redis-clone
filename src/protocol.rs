/// Implementation of the RESP protool https://redis.io/topics/protocol
use std::cmp::PartialEq;
use std::convert::{TryFrom, TryInto};
use std::io::{BufRead, Read};

use crate::errors::{ProtoError, Result};

const DEPTH_LIMIT: usize = 512;
const MAX_ARRAY_SIZE: usize = 1024 * 1024;
const MAX_BULK_STR_SIZE: usize = 512 * 1024 * 1024;
const MAX_LINE_LENGTH: usize = 64 * 1024;
const LF: u8 = b'\n';
const CRLF: &[u8] = b"\r\n";

#[derive(Debug, PartialEq, Eq)]
pub enum RObj {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Option<Vec<RObj>>),
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
enum RSym {
    SimpleString = b'+',
    Error = b'-',
    Integer = b':',
    BulkString = b'$',
    Array = b'*',
}

impl TryFrom<u8> for RSym {
    type Error = ProtoError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            v if v == Self::SimpleString as u8 => Ok(Self::SimpleString),
            v if v == Self::Error as u8 => Ok(Self::Error),
            v if v == Self::Integer as u8 => Ok(Self::Integer),
            v if v == Self::BulkString as u8 => Ok(Self::BulkString),
            v if v == Self::Array as u8 => Ok(Self::Array),
            _ => Err(ProtoError::UnsupportedSymbol),
        }
    }
}

pub fn decode<T: BufRead>(mut stream: T) -> Result<RObj> {
    do_decode(&mut stream, 0)
}

fn do_decode(stream: &mut impl BufRead, depth: usize) -> Result<RObj> {
    if depth > DEPTH_LIMIT {
        return Err(ProtoError::ExceededDepthLimit);
    }

    let (type_sym, value_str) = read_header(stream)?;

    let value = match type_sym {
        RSym::SimpleString => RObj::SimpleString(value_str.into()),
        RSym::Error => RObj::Error(value_str.into()),
        RSym::Integer => RObj::Integer(value_str.parse()?),
        RSym::BulkString => {
            let len = value_str
                .parse()
                .or(Err(ProtoError::InvalidBulkStringSize))?;
            let value = read_bulk_string(stream, len)?;
            RObj::BulkString(value)
        }
        RSym::Array => {
            let len = value_str.parse().or(Err(ProtoError::InvalidArraySize))?;
            let value = read_array(stream, len, depth)?;
            RObj::Array(value)
        }
    };

    Ok(value)
}

fn read_header(stream: &mut impl BufRead) -> Result<(RSym, String)> {
    let mut buffer = vec![];
    read_line(stream, &mut buffer)?;

    let (&type_sym, tail) = buffer
        .split_first()
        .ok_or_else(|| ProtoError::from("Error parsing resp header structure"))?;

    let type_sym = RSym::try_from(type_sym)?;
    let tail = std::str::from_utf8(tail)?.to_owned();

    Ok((type_sym, tail))
}

fn read_line(stream: &mut impl BufRead, buffer: &mut Vec<u8>) -> Result<()> {
    let limit = MAX_LINE_LENGTH.try_into().unwrap();
    let num_bytes = stream.by_ref().take(limit).read_until(LF, buffer)?;

    // If we got nothing then we can assume the connection has closed
    if num_bytes == 0 {
        return Err(ProtoError::ConnectionClosed);
    }
    // We must have at least 2 bytes for CRLF
    if num_bytes < 2 {
        return Err(ProtoError::InvalidTerminator);
    }
    // The line must be terminated by CRLF
    if &buffer[(num_bytes - 2)..] != CRLF {
        // We may be missing the CRLF because the line limit has been exceeded
        if num_bytes == MAX_LINE_LENGTH {
            return Err(ProtoError::ExceededMaxLineLength);
        }

        return Err(ProtoError::InvalidTerminator);
    }

    // Drop the CRLF
    buffer.truncate(num_bytes - 2);

    Ok(())
}

fn read_bulk_string(stream: &mut impl BufRead, len: i64) -> Result<Option<String>> {
    if len == -1 {
        return Ok(None);
    }

    let len = usize::try_from(len).or(Err(ProtoError::InvalidBulkStringSize))?;

    if len > MAX_BULK_STR_SIZE {
        return Err(ProtoError::InvalidBulkStringSize);
    }

    let mut buffer = vec![0; len + 2];
    stream.read_exact(&mut buffer)?;
    let value_str = std::str::from_utf8(&buffer[..len])?;

    Ok(Some(value_str.to_owned()))
}

fn read_array(stream: &mut impl BufRead, len: i64, depth: usize) -> Result<Option<Vec<RObj>>> {
    if len == -1 {
        return Ok(None);
    }

    let len = usize::try_from(len).or(Err(ProtoError::InvalidArraySize))?;

    if len > MAX_ARRAY_SIZE {
        return Err(ProtoError::InvalidArraySize);
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
            ProtoError::ConnectionClosed
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
        assert_eq!(result.unwrap_err(), ProtoError::InvalidTerminator);

        // Invalid case: a single LF preceeded by something other than a CR
        let mut buffer = vec![];
        let mut input: &[u8] = b"x\n";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.unwrap_err(), ProtoError::InvalidTerminator);

        // Invalid case: a single CR followed by not a LF
        let mut buffer = vec![];
        let mut input: &[u8] = b"\rx";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.unwrap_err(), ProtoError::InvalidTerminator);

        // Invalid case: no terminator
        let mut buffer = vec![];
        let mut input: &[u8] = b"x";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.unwrap_err(), ProtoError::InvalidTerminator);
    }

    #[test]
    fn test_read_line_gt_max() {
        let input: Vec<u8> = b"0".repeat(MAX_LINE_LENGTH + 1);

        let result = decode(input.as_slice());
        assert_eq!(result.unwrap_err(), ProtoError::ExceededMaxLineLength);
    }

    #[test]
    fn rsym() {
        assert_eq!(RSym::try_from(b'+').unwrap(), RSym::SimpleString);
        assert_eq!(RSym::try_from(b'-').unwrap(), RSym::Error);
        assert_eq!(RSym::try_from(b':').unwrap(), RSym::Integer);
        assert_eq!(RSym::try_from(b'$').unwrap(), RSym::BulkString);
        assert_eq!(RSym::try_from(b'*').unwrap(), RSym::Array);
        assert_eq!(
            RSym::try_from(b'0').unwrap_err(),
            ProtoError::UnsupportedSymbol
        );
    }

    #[test]
    fn decode_simple_string() {
        let input: &[u8] = b"+OK\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RObj::SimpleString("OK".into()));
    }

    #[test]
    fn decode_error() {
        let input: &[u8] = b"-Error message\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RObj::Error("Error message".into()));
    }

    #[test]
    fn decode_integer() {
        let input: &[u8] = b":1000\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RObj::Integer(1000i64));
    }

    #[test]
    fn decode_bulk_string() {
        let input: &[u8] = b"$8\r\nabc\r\ndef\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RObj::BulkString(Some("abc\r\ndef".into())));
    }

    #[test]
    fn decode_empty_bulk_string() {
        let input: &[u8] = b"$0\r\n\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RObj::BulkString(Some("".into())));
    }

    #[test]
    fn decode_null_bulk_string() {
        let input: &[u8] = b"$-1\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RObj::BulkString(None));
    }

    #[test]
    fn decode_null_array() {
        let input: &[u8] = b"*-1\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RObj::Array(None));
    }

    #[test]
    fn decode_empty_array() {
        let input: &[u8] = b"*0\r\n";
        let result = decode(input);
        assert_eq!(result.unwrap(), RObj::Array(Some(vec![])));
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
            RObj::Array(Some(vec![
                RObj::SimpleString("1".into()),
                RObj::Error("2".into()),
                RObj::Integer(3),
                RObj::BulkString(Some("4".into())),
                RObj::Array(Some(vec![RObj::Array(Some(vec![RObj::Array(None),])),]))
            ]))
        );
    }

    #[test]
    fn limit_recursion() {
        // Recurse beyond the limit
        let input: Vec<u8> = b"*1\r\n".repeat(DEPTH_LIMIT + 1);

        let result = decode(input.as_slice());
        assert_eq!(result.unwrap_err(), ProtoError::ExceededDepthLimit);
    }

    #[test]
    fn array_invalid_size_overflow() {
        // i64 max + 1
        let input: &[u8] = b"*9223372036854775808\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), ProtoError::InvalidArraySize);
    }

    #[test]
    fn array_invalid_size_negative() {
        let input: &[u8] = b"*-2\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), ProtoError::InvalidArraySize);
    }

    #[test]
    fn array_invalid_size_gt_max() {
        // 1024 * 1024 + 1 is too large
        let input: &[u8] = b"*1048577\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), ProtoError::InvalidArraySize);
    }

    #[test]
    fn bulk_string_invalid_size_overflow() {
        // i64 max + 1
        let input: &[u8] = b"$9223372036854775808\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), ProtoError::InvalidBulkStringSize);
    }

    #[test]
    fn bulk_string_invalid_size_negative() {
        let input: &[u8] = b"$-2\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), ProtoError::InvalidBulkStringSize);
    }

    #[test]
    fn bulk_string_invalid_size_gt_max() {
        // 512 * 1024 * 1024 + 1 is too large
        let input: &[u8] = b"$536870913\r\n";

        let result = decode(input);
        assert_eq!(result.unwrap_err(), ProtoError::InvalidBulkStringSize);
    }
}
