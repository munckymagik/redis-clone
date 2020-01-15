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
    pub command: String,
    argv: Vec<String>,
}

impl Request {
    pub fn maybe_arg(&self, index: usize) -> Option<&String> {
        self.argv.get(index)
    }

    pub fn arg(&self, index: usize) -> Result<&String> {
        self.maybe_arg(index).ok_or_else(|| {
            let msg = format!("Argument at {} does not exist", index);
            Error::from(msg)
        })
    }

    pub fn arity(&self) -> i64 {
        (1 + self.argv.len()).try_into().unwrap()
    }

    pub fn arguments(&self) -> &[String] {
        &self.argv
    }

    pub fn argv_to_string(&self) -> String {
        self.argv
            .iter()
            .map(|v| format!("`{}`,", v))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl TryFrom<Vec<String>> for Request {
    type Error = Error;

    fn try_from(mut query: Vec<String>) -> Result<Self> {
        if query.len() == 0 {
            return Err(Error::EmptyRequest);
        }

        let tail = query.drain(1..).collect();
        let head = query.pop().unwrap();

        Ok(Request {
            command: head,
            argv: tail,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_into_request() {
        let input = vec!["set".to_string(), "x".to_string(), "1".to_string()];
        let request = Request::try_from(input).unwrap();

        assert_eq!(request.command, "set");
        assert_eq!(request.argv, &["x", "1"]);
    }

    #[test]
    fn test_try_into_request_no_args() {
        let input = vec!["set".to_string()];
        let request = Request::try_from(input).unwrap();

        assert_eq!(request.command, "set");
        assert_eq!(request.argv, &[] as &[&str]);
    }

    #[test]
    fn test_try_into_request_error_for_empty_array() {
        let input = vec![];
        let output = Request::try_from(input);
        assert_eq!(output.unwrap_err(), Error::EmptyRequest);
    }

    #[test]
    fn test_args_to_string() {
        let request = Request {
            command: "xxx".to_owned(),
            argv: vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
        };

        // Note: final comma is for consistency with real Redis
        assert_eq!(request.argv_to_string(), "`1`, `2`, `3`,");
    }

    #[test]
    fn test_arity() {
        let request = Request {
            command: "xxx".to_owned(),
            argv: vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
        };

        assert_eq!(request.arity(), 4);
    }
}
