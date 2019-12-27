use crate::{
    errors::{Error, Result},
    protocol::RespVal,
};
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Request {
    pub command: String,
    pub argv: Vec<String>,
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
}
