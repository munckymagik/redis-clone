//! Parse "RESP Arrays of Bulk Strings" as defined in the the RESP protocol
//! documentation here: https://redis.io/topics/protocol

mod errors;

pub use errors::{ProtoError, ProtoResult};

const MAX_ARRAY_SIZE: usize = 1024 * 1024;
const MAX_BULK_STR_SIZE: usize = 512 * 1024 * 1024;
const MAX_LINE_LENGTH: usize = 64 * 1024;
const LF: u8 = b'\n';
const CRLF: &[u8] = b"\r\n";

use std::convert::{TryFrom, TryInto};
use std::marker::Unpin;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

pub async fn decode<T: AsyncBufRead + Unpin + Send>(mut stream: T) -> ProtoResult<Vec<String>> {
    let (type_sym, value_str) = read_header(&mut stream).await?;

    if type_sym != b'*' {
        return Err(ProtoError::UnsupportedSymbol(type_sym.into()));
    }

    let len = value_str.parse().or(Err(ProtoError::InvalidArraySize))?;
    let value = read_array(&mut stream, len).await?;
    Ok(value)
}

async fn read_header(stream: &mut (impl AsyncBufRead + Unpin + Send)) -> ProtoResult<(u8, String)> {
    let mut buffer = vec![];
    read_line(stream, &mut buffer).await?;

    let (&type_sym, tail) = buffer
        .split_first()
        .ok_or_else(|| ProtoError::from("Error parsing resp header structure"))?;

    let tail = std::str::from_utf8(tail)?.to_owned();

    Ok((type_sym, tail))
}

async fn read_line(
    stream: &mut (impl AsyncBufRead + Unpin + Send),
    buffer: &mut Vec<u8>,
) -> ProtoResult<()> {
    let limit = MAX_LINE_LENGTH.try_into().unwrap();
    let num_bytes = stream.take(limit).read_until(LF, buffer).await?;

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

async fn read_bulk_string(
    stream: &mut (impl AsyncBufRead + Unpin + Send),
    len: i64,
) -> ProtoResult<String> {
    let len = usize::try_from(len).or(Err(ProtoError::InvalidBulkStringSize))?;

    if len > MAX_BULK_STR_SIZE {
        return Err(ProtoError::InvalidBulkStringSize);
    }

    let mut buffer = vec![0; len + 2];
    stream.read_exact(&mut buffer).await?;
    let value_str = std::str::from_utf8(&buffer[..len])?;

    Ok(value_str.to_owned())
}

async fn read_array(
    stream: &mut (impl AsyncBufRead + Unpin + Send),
    len: i64,
) -> ProtoResult<Vec<String>> {
    // We don't need to support empty or null arrays in requests
    if len == 0 || len == -1 {
        return Err(ProtoError::EmptyRequest);
    }

    let len = usize::try_from(len).or(Err(ProtoError::InvalidArraySize))?;

    if len > MAX_ARRAY_SIZE {
        return Err(ProtoError::InvalidArraySize);
    }

    let mut elements = Vec::with_capacity(len);

    for _ in 0..len {
        let (type_sym, value_str) = read_header(stream).await?;
        if type_sym != b'$' {
            return Err(ProtoError::UnsupportedSymbol(type_sym.into()));
        }

        let len = value_str
            .parse()
            .or(Err(ProtoError::InvalidBulkStringSize))?;
        let value = read_bulk_string(stream, len).await?;

        elements.push(value);
    }

    Ok(elements)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_read_line() {
        use std::io::Cursor;
        use std::io::Write;

        let mut stream = Cursor::new(Vec::new());
        stream.write_all(b"123\r\n456\r\n").unwrap();
        stream.set_position(0);
        let mut buffer = vec![];

        read_line(&mut stream, &mut buffer).await.unwrap();
        assert_eq!(buffer, b"123");

        buffer.clear();
        read_line(&mut stream, &mut buffer).await.unwrap();
        assert_eq!(buffer, b"456");

        buffer.clear();
        assert_eq!(
            read_line(&mut stream, &mut buffer).await.unwrap_err(),
            ProtoError::ConnectionClosed
        );
    }

    #[tokio::test]
    async fn test_read_line_invalid_terminator() {
        // Valid case: an empty line
        let mut buffer = vec![];
        let mut input: &[u8] = b"\r\n";
        read_line(&mut input, &mut buffer).await.unwrap();
        assert_eq!(buffer, b"");

        // Invalid case: a single LF without a CR
        let mut buffer = vec![];
        let mut input: &[u8] = b"\n";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.await.unwrap_err(), ProtoError::InvalidTerminator);

        // Invalid case: a single LF preceeded by something other than a CR
        let mut buffer = vec![];
        let mut input: &[u8] = b"x\n";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.await.unwrap_err(), ProtoError::InvalidTerminator);

        // Invalid case: a single CR followed by not a LF
        let mut buffer = vec![];
        let mut input: &[u8] = b"\rx";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.await.unwrap_err(), ProtoError::InvalidTerminator);

        // Invalid case: no terminator
        let mut buffer = vec![];
        let mut input: &[u8] = b"x";
        let result = read_line(&mut input, &mut buffer);
        assert_eq!(result.await.unwrap_err(), ProtoError::InvalidTerminator);
    }

    #[tokio::test]
    async fn test_read_line_gt_max() {
        let mut buffer = vec![];
        let input: Vec<u8> = b"0".repeat(MAX_LINE_LENGTH + 1);
        let mut slice = input.as_slice();

        let result = read_line(&mut slice, &mut buffer);
        assert_eq!(result.await.unwrap_err(), ProtoError::ExceededMaxLineLength);
    }

    #[tokio::test]
    async fn decode_not_an_array() {
        let input: &[u8] = b"x\r\n";
        let result = decode(input);
        assert_eq!(result.await.unwrap_err(), ProtoError::UnsupportedSymbol('x'));
    }

    #[tokio::test]
    async fn decode_null_array() {
        let input: &[u8] = b"*-1\r\n";
        let result = decode(input);
        assert_eq!(result.await.unwrap_err(), ProtoError::EmptyRequest);
    }

    #[tokio::test]
    async fn decode_empty_array() {
        let input: &[u8] = b"*0\r\n";
        let result = decode(input);
        assert_eq!(result.await.unwrap_err(), ProtoError::EmptyRequest);
    }

    #[tokio::test]
    async fn decode_array_of_bulk_string() {
        let input: &[u8] = b"*2\r\n$8\r\nabc\r\ndef\r\n$3\r\n123\r\n";
        let result = decode(input);
        assert_eq!(
            result.await.unwrap(),
            vec!["abc\r\ndef".to_string(), "123".to_string()]
        );
    }

    #[tokio::test]
    async fn decode_array_of_not_bulk_string() {
        let input: &[u8] = b"*1\r\n:1\r\n";
        let result = decode(input);
        assert_eq!(result.await.unwrap_err(), ProtoError::UnsupportedSymbol(':'));
    }

    #[tokio::test]
    async fn decode_empty_bulk_string() {
        let input: &[u8] = b"*1\r\n$0\r\n\r\n";
        let result = decode(input);
        assert_eq!(result.await.unwrap(), vec!["".to_string()]);
    }

    #[tokio::test]
    async fn array_invalid_size_overflow() {
        // i64 max + 1
        let input: &[u8] = b"*9223372036854775808\r\n";

        let result = decode(input);
        assert_eq!(result.await.unwrap_err(), ProtoError::InvalidArraySize);
    }

    #[tokio::test]
    async fn array_invalid_size_negative() {
        let input: &[u8] = b"*-2\r\n";

        let result = decode(input);
        assert_eq!(result.await.unwrap_err(), ProtoError::InvalidArraySize);
    }

    #[tokio::test]
    async fn array_invalid_size_gt_max() {
        // 1024 * 1024 + 1 is too large
        let input: &[u8] = b"*1048577\r\n";

        let result = decode(input);
        assert_eq!(result.await.unwrap_err(), ProtoError::InvalidArraySize);
    }

    #[tokio::test]
    async fn bulk_string_invalid_size_overflow() {
        // i64 max + 1
        let input: &[u8] = b"*1\r\n$9223372036854775808\r\n";

        let result = decode(input);
        assert_eq!(result.await.unwrap_err(), ProtoError::InvalidBulkStringSize);
    }

    #[tokio::test]
    async fn bulk_string_invalid_size_negative() {
        // We don't need to support null strings in requests (I think) so we
        // consider the -1 "null string" marker as invalid
        let input: &[u8] = b"*1\r\n$-1\r\n";

        let result = decode(input);
        assert_eq!(result.await.unwrap_err(), ProtoError::InvalidBulkStringSize);
    }

    #[tokio::test]
    async fn bulk_string_invalid_size_gt_max() {
        // 512 * 1024 * 1024 + 1 is too large
        let input: &[u8] = b"*1\r\n$536870913\r\n";

        let result = decode(input);
        assert_eq!(result.await.unwrap_err(), ProtoError::InvalidBulkStringSize);
    }
}
