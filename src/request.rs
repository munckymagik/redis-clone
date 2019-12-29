use crate::{
    errors::{Error, Result},
    protocol,
};
use std::convert::TryFrom;
use std::marker::Unpin;
use tokio::io::AsyncBufRead;

pub async fn parse(stream: &mut (impl AsyncBufRead + Unpin + Send)) -> Result<Request> {
    let query = protocol::decode2(stream).await?;
    Request::try_from(query)
}

#[derive(Debug, PartialEq)]
pub struct Request {
    pub command: String,
    argv: Vec<String>,
}

impl Request {
    #[cfg(test)]
    pub fn new(name: &str, argv: &[&str]) -> Self {
        Self {
            command: name.to_owned(),
            argv: argv.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn arg(&self, index: usize) -> Option<&String> {
        self.argv.get(index)
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
            return Err(Error::EmptyQuery);
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
        assert_eq!(output.unwrap_err(), Error::EmptyQuery);
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
}
