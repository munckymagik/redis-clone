use std::convert::TryInto;
use std::io::{BufRead, Read};

use super::{RespError, RespResult, CRLF, LF, MAX_LINE_LENGTH};

pub(super) fn read_line(stream: &mut impl BufRead, buffer: &mut Vec<u8>) -> RespResult<()> {
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
        let mut buffer = vec![];
        let input: Vec<u8> = b"0".repeat(MAX_LINE_LENGTH + 1);

        let result = read_line(&mut input.as_slice(), &mut buffer);
        assert_eq!(result.unwrap_err(), RespError::ExceededMaxLineLength);
    }
}
