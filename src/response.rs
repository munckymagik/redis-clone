use std::fmt::Display;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RespSym {
    SimpleString = b'+',
    Error = b'-',
    Integer = b':',
    BulkString = b'$',
    Array = b'*',
}

use self::RespSym::*;

impl RespSym {
    pub fn as_ascii(self) -> u8 {
        self as u8
    }
}

#[derive(Debug)]
pub struct Response {
    buffer: Vec<u8>,
}

impl Response {
    pub fn new() -> Self {
        Self { buffer: vec![] }
    }

    pub fn add_array_len(&mut self, len: i64) {
        self.add(Array, len);
    }

    pub fn add_null_array(&mut self) {
        self.add(Array, -1i64);
    }

    pub fn add_simple_string(&mut self, value: &str) {
        self.add(SimpleString, value);
    }

    pub fn add_bulk_string(&mut self, value: &str) {
        self.add(BulkString, value.len());
        self.buffer.extend(value.as_bytes());
        self.buffer.extend(b"\r\n");
    }

    pub fn add_null_string(&mut self) {
        self.add(BulkString, -1i64);
    }

    pub fn add_error(&mut self, value: &str) {
        self.add(Error, value);
    }

    pub fn add_integer(&mut self, value: i64) {
        self.add(Integer, value);
    }

    fn add(&mut self, sym: RespSym, value: impl Display) {
        self.buffer.push(sym.as_ascii());
        self.buffer.extend(format!("{}", value).as_bytes());
        self.buffer.extend(b"\r\n");
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.buffer.as_ref()
    }

    #[cfg(test)]
    pub fn as_string(&self) -> String {
        std::str::from_utf8(&self.buffer).expect("utf8 error").to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rsym() {
        assert_eq!(RespSym::Array.as_ascii(), b'*');
    }

    #[test]
    fn test_array() {
        let mut builder = Response::new();
        builder.add_array_len(1);
        builder.add_array_len(2);
        builder.add_integer(23);
        builder.add_simple_string("x y z");

        let expected = b"\
                *1\r\n\
                *2\r\n\
                :23\r\n\
                +x y z\r\n";

        assert_eq!(builder.as_bytes(), expected);
    }

    #[test]
    fn test_integer() {
        let mut builder = Response::new();
        builder.add_integer(23);
        assert_eq!(builder.as_bytes(), b":23\r\n");
    }

    #[test]
    fn test_simple_string() {
        let mut builder = Response::new();
        builder.add_simple_string("x y z");
        assert_eq!(builder.as_bytes(), b"+x y z\r\n");
    }

    #[test]
    fn test_bulk_string() {
        let mut builder = Response::new();
        builder.add_bulk_string("x\ny\nz");
        let expected = b"\
            $5\r\n\
            x\ny\nz\r\n";
        assert_eq!(builder.as_bytes(), expected);
    }

    #[test]
    fn test_null_array() {
        let mut builder = Response::new();
        builder.add_null_array();
        assert_eq!(builder.as_bytes(), b"*-1\r\n");
    }

    #[test]
    fn test_null_string() {
        let mut builder = Response::new();
        builder.add_null_string();
        assert_eq!(builder.as_bytes(), b"$-1\r\n");
    }

    #[test]
    fn test_error() {
        let mut builder = Response::new();
        builder.add_error("ERR poop detected");
        assert_eq!(builder.as_bytes(), b"-ERR poop detected\r\n");
    }
}
