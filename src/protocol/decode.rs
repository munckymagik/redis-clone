use std::convert::TryFrom;
use std::io::BufRead;

use super::{
    io::read_line, RespError, RespResult, RespSym, RespVal, DEPTH_LIMIT, MAX_ARRAY_SIZE,
    MAX_BULK_STR_SIZE,
};

pub fn decode<T: BufRead>(mut stream: T) -> RespResult<RespVal> {
    do_decode(&mut stream, 0)
}

fn do_decode(stream: &mut impl BufRead, depth: usize) -> RespResult<RespVal> {
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

fn read_header(stream: &mut impl BufRead) -> RespResult<(RespSym, String)> {
    let mut buffer = vec![];
    read_line(stream, &mut buffer)?;

    let (&type_sym, tail) = buffer
        .split_first()
        .ok_or_else(|| RespError::from("Error parsing resp header structure"))?;

    let type_sym = RespSym::try_from(type_sym)?;
    let tail = std::str::from_utf8(tail)?.to_owned();

    Ok((type_sym, tail))
}

fn read_bulk_string(stream: &mut impl BufRead, len: i64) -> RespResult<Option<String>> {
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

fn read_array(
    stream: &mut impl BufRead,
    len: i64,
    depth: usize,
) -> RespResult<Option<Vec<RespVal>>> {
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