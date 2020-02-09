use crate::{
    errors::{Error, Result},
    protocol,
};
use std::convert::{TryFrom, TryInto};
use std::marker::Unpin;
use tokio::io::AsyncBufRead;

pub async fn parse(stream: &mut (impl AsyncBufRead + Unpin + Send)) -> Result<Request> {
    let query = protocol::decode(stream).await?;
    Request::try_from(query)
}

#[derive(Debug, PartialEq)]
pub struct Request {
    argv: Vec<String>,
}

impl Request {
    pub fn maybe_arg(&self, index: usize) -> Option<&String> {
        self.argv.get(index + 1)
    }

    pub fn command(&self) -> &str {
        &self.argv[0]
    }

    pub fn arg(&self, index: usize) -> Result<&String> {
        self.maybe_arg(index).ok_or_else(|| {
            let msg = format!("Argument at {} does not exist", index);
            Error::from(msg)
        })
    }

    pub fn arity(&self) -> i64 {
        self.argv.len().try_into().unwrap()
    }

    pub fn arguments(&self) -> &[String] {
        &self.argv[1..]
    }

    pub fn argv_to_string(&self) -> String {
        self.argv[1..]
            .iter()
            .map(|v| format!("`{}`,", v))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl TryFrom<Vec<Vec<u8>>> for Request {
    type Error = Error;

    fn try_from(query: Vec<Vec<u8>>) -> Result<Self> {
        use std::result::Result as StdResult;

        if query.is_empty() {
            return Err(Error::EmptyRequest);
        }

        let maybe_argv: StdResult<Vec<String>, _> = query
            .into_iter()
            .map(|v| String::from_utf8(v))
            .collect();

        let argv = maybe_argv?;

        Ok(Request {
            argv,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_into_request() {
        let input = vec![b"set".to_vec(), b"x".to_vec(), b"1".to_vec()];
        let request = Request::try_from(input).unwrap();

        assert_eq!(request.command(), "set");
        assert_eq!(request.arguments(), &["x", "1"]);
    }

    #[test]
    fn test_try_into_request_no_args() {
        let input = vec![b"set".to_vec()];
        let request = Request::try_from(input).unwrap();

        assert_eq!(request.command(), "set");
        assert_eq!(request.arguments(), &[] as &[&str]);
    }

    #[test]
    fn test_try_into_request_error_for_empty_array() {
        let input = vec![];
        let output = Request::try_from(input);
        assert_eq!(output.unwrap_err(), Error::EmptyRequest);
    }

    #[test]
    fn test_args_to_string() {
        let request = Request::try_from(vec![b"xxx".to_vec(), b"1".to_vec(), b"2".to_vec(), b"3".to_vec()]).unwrap();

        // Note: final comma is for consistency with real Redis
        assert_eq!(request.argv_to_string(), "`1`, `2`, `3`,");
    }

    #[test]
    fn test_arity() {
        let request = Request::try_from(vec![b"xxx".to_vec(), b"1".to_vec(), b"2".to_vec(), b"3".to_vec()]).unwrap();

        assert_eq!(request.arity(), 4);
    }

    #[test]
    fn test_maybe_arg() {
        let request = Request::try_from(vec![b"xxx".to_vec(), b"1".to_vec()]).unwrap();

        assert_eq!(request.maybe_arg(0), Some(&"1".to_string()));
        assert_eq!(request.maybe_arg(1), None);
    }

    #[test]
    fn test_arg() {
        let request = Request::try_from(vec![b"xxx".to_vec(), b"1".to_vec()]).unwrap();

        assert_eq!(request.arg(0), Ok(&"1".to_string()));
        assert_eq!(request.arg(1), Err(Error::from("Argument at 1 does not exist")));
    }
}
