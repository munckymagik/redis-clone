use crate::protocol::{
    self, RespResult,
    RespSym::{self, *},
    RespVal,
};
use std::fmt::Display;

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

    pub fn decode(&self) -> RespResult<RespVal> {
        let bytes = self.as_bytes();
        protocol::decode(bytes.as_slice())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder() {
        let mut builder = Response::new();
        builder.add_array_len(4);
        builder.add_integer(23);
        builder.add_simple_string("x y z");
        builder.add_bulk_string("x\ny\nz");
        builder.add_error("ERR poop detected");

        let expected = b"\
                *4\r\n\
                :23\r\n\
                +x y z\r\n\
                $5\r\n\
                x\ny\nz\r\n\
                -ERR poop detected\r\n"
            .to_vec();

        assert_eq!(builder.as_bytes(), expected);
    }
}
