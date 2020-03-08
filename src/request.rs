use crate::{
    errors::{Error, Result},
    protocol,
};
use byte_string::{ByteStr, ByteString};
use std::convert::{TryFrom, TryInto};
use std::marker::Unpin;
use tokio::io::AsyncBufRead;

pub async fn parse(stream: &mut (impl AsyncBufRead + Unpin + Send)) -> Result<Request> {
    let query = protocol::decode(stream).await?;
    Request::try_from(query)
}

#[derive(Debug, PartialEq)]
pub struct Request {
    query: Vec<ByteString>,
}

impl Request {
    pub fn command(&self) -> ByteStr {
        self.query[0].as_byte_str()
    }

    pub fn maybe_arg(&self, index: usize) -> Option<&ByteString> {
        self.query.get(index + 1)
    }

    pub fn arg(&self, index: usize) -> Result<&ByteString> {
        self.maybe_arg(index).ok_or_else(|| {
            let msg = format!("Argument at {} does not exist", index);
            Error::from(msg)
        })
    }

    pub fn arity(&self) -> i64 {
        self.query.len().try_into().unwrap()
    }

    pub fn arguments(&self) -> &[ByteString] {
        &self.query[1..]
    }

    pub fn argv_to_string(&self) -> String {
        self.query[1..]
            .iter()
            .map(|v| format!("`{}`,", v))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl TryFrom<Vec<ByteString>> for Request {
    type Error = Error;

    fn try_from(query: Vec<ByteString>) -> Result<Self> {
        if query.is_empty() {
            return Err(Error::EmptyRequest);
        }

        Ok(Request { query })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_into_request() {
        let input = vec!["set".into(), "x".into(), "1".into()];
        let request = Request::try_from(input).unwrap();

        assert_eq!(request.command(), ByteStr::from("set"));
        assert_eq!(request.arguments(), &["x".into(), "1".into()]);
    }

    #[test]
    fn test_try_into_request_no_args() {
        let input = vec!["set".into()];
        let request = Request::try_from(input).unwrap();

        assert_eq!(request.command(), ByteStr::from("set"));
        assert_eq!(request.arguments(), &[] as &[ByteString]);
    }

    #[test]
    fn test_try_into_request_error_for_empty_array() {
        let input = vec![];
        let output = Request::try_from(input);
        assert_eq!(output.unwrap_err(), Error::EmptyRequest);
    }

    #[test]
    fn test_args_to_string() {
        let request =
            Request::try_from(vec!["xxx".into(), "1".into(), "2".into(), "3".into()]).unwrap();

        // Note: final comma is for consistency with real Redis
        assert_eq!(request.argv_to_string(), "`1`, `2`, `3`,");
    }

    #[test]
    fn test_arity() {
        let request =
            Request::try_from(vec!["xxx".into(), "1".into(), "2".into(), "3".into()]).unwrap();

        assert_eq!(request.arity(), 4);
    }

    #[test]
    fn test_maybe_arg() {
        let request = Request::try_from(vec!["xxx".into(), "1".into()]).unwrap();

        assert_eq!(request.maybe_arg(0), Some(&ByteString::from("1")));
        assert_eq!(request.maybe_arg(1), None);
    }

    #[test]
    fn test_arg() {
        let request = Request::try_from(vec!["xxx".into(), "1".into()]).unwrap();

        assert_eq!(request.arg(0), Ok(&ByteString::from("1")));
        assert_eq!(
            request.arg(1),
            Err(Error::from("Argument at 1 does not exist"))
        );
    }
}
