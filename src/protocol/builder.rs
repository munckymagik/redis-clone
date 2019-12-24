use super::{decode, RespResult, RespVal};

pub struct RespBuilder {
    lines: Vec<String>,
}

impl RespBuilder {
    pub fn new() -> Self {
        Self { lines: vec![] }
    }

    pub fn add_array_len(&mut self, len: i64) {
        self.lines.push(format!("*{}", len));
    }

    pub fn add_simple_string(&mut self, value: &str) {
        self.lines.push(format!("+{}", value));
    }

    pub fn add_bulk_string(&mut self, value: &str) {
        self.lines.push(format!("${}", value.len()));
        self.lines.push(value.to_owned());
    }

    pub fn add_error(&mut self, value: &str) {
        self.lines.push(format!("-{}", value));
    }

    pub fn add_integer(&mut self, value: i64) {
        self.lines.push(format!(":{}", value));
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buffer = self.lines.join("\r\n");
        buffer.push_str("\r\n");
        buffer.into_bytes()
    }

    pub fn decode(&self) -> RespResult<RespVal> {
        let bytes = self.as_bytes();
        decode(bytes.as_slice())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_builder() {
        let mut builder = RespBuilder::new();
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
