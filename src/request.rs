use crate::{
    errors::{Error, Result},
    protocol::{self, RespVal},
};
use std::convert::TryFrom;
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

impl TryFrom<RespVal> for Request {
    type Error = Error;

    fn try_from(query: RespVal) -> Result<Self> {
        let args = match query {
            RespVal::Array(Some(args)) => args,
            RespVal::Array(None) => vec![],
            _ => return Err(Error::UnsupportedRequestType),
        };

        if args.is_empty() {
            return Err(Error::EmptyQuery);
        }

        let mut flattened = vec![];

        for v in args {
            match v {
                RespVal::BulkString(Some(arg)) => flattened.push(arg),
                _ => return Err(Error::ProtocolError),
            }
        }

        let (cmd, args) = flattened.split_first().unwrap();
        Ok(Request {
            command: cmd.to_owned(),
            argv: args.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_into_request() {
        let input = RespVal::Array(Some(vec![
            RespVal::BulkString(Some("set".into())),
            RespVal::BulkString(Some("x".into())),
            RespVal::BulkString(Some("1".into())),
        ]));

        let request = Request::try_from(input).unwrap();
        assert_eq!(request.command, "set");
        assert_eq!(request.argv, &["x", "1"]);
    }

    #[test]
    fn test_try_into_request_invalid_request_type() {
        let out = Request::try_from(RespVal::Integer(1));
        assert_eq!(out.unwrap_err(), Error::UnsupportedRequestType);
    }

    #[test]
    fn test_try_into_request_not_multi_bulk() {
        let input = RespVal::Array(Some(vec![
            RespVal::BulkString(Some("set".into())),
            RespVal::BulkString(Some("x".into())),
            RespVal::Integer(1),
        ]));
        let out = Request::try_from(input);
        assert_eq!(out.unwrap_err(), Error::ProtocolError);
    }

    #[test]
    fn test_try_into_request_empty_array() {
        let input = RespVal::Array(Some(vec![]));
        let out = Request::try_from(input);
        assert_eq!(out.unwrap_err(), Error::EmptyQuery);

        let input = RespVal::Array(None);
        let out = Request::try_from(input);
        assert_eq!(out.unwrap_err(), Error::EmptyQuery);
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
