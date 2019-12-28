use crate::protocol::RespSym::{self, *};
use std::fmt::Display;

#[cfg(test)]
use crate::{
    errors::{Error, Result},
    protocol::{self, RespVal},
};

#[derive(Debug)]
pub struct Response {
    lines: Vec<String>,
}

impl Response {
    pub fn new() -> Self {
        Self { lines: vec![] }
    }

    pub fn add_array_len(&mut self, len: i64) {
        self.add(Array, len);
    }

    pub fn add_simple_string(&mut self, value: &str) {
        self.add(SimpleString, value);
    }

    pub fn add_bulk_string(&mut self, value: &str) {
        self.add(BulkString, value.len());
        self.add_line(value);
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
        self.add_line(&format!("{}{}", sym.as_char(), value));
    }

    fn add_line(&mut self, value: &str) {
        self.lines.push(format!("{}\r\n", value));
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.lines.join("").into_bytes()
    }

    #[cfg(test)]
    pub async fn decode(&self) -> Result<RespVal> {
        let bytes = self.as_bytes();
        protocol::decode(bytes.as_slice())
            .await
            .map_err(|e| Error::from(e))
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
