mod commands;
mod errors;
pub mod protocol;

pub use commands::lookup_command;
pub use errors::Error;
use protocol::RespVal;

#[derive(Debug, PartialEq)]
pub struct MultiCmd {
    pub command: String,
    pub argv: Vec<String>,
}

pub fn process_query(query: RespVal) -> Result<MultiCmd, Error> {
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
    Ok(MultiCmd {
        command: cmd.to_owned(),
        argv: args.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_query() {
        let input = RespVal::Array(Some(vec![
            RespVal::BulkString(Some("set".into())),
            RespVal::BulkString(Some("x".into())),
            RespVal::BulkString(Some("1".into())),
        ]));

        let multi_cmd = process_query(input).unwrap();
        assert_eq!(multi_cmd.command, "set");
        assert_eq!(multi_cmd.argv, &["x", "1"]);
    }

    #[test]
    fn test_process_query_invalid_request_type() {
        let out = process_query(RespVal::Integer(1));
        assert_eq!(out.unwrap_err(), Error::UnsupportedRequestType);
    }

    #[test]
    fn test_process_query_not_multi_bulk() {
        let input = RespVal::Array(Some(vec![
            RespVal::BulkString(Some("set".into())),
            RespVal::BulkString(Some("x".into())),
            RespVal::Integer(1),
        ]));
        let out = process_query(input);
        assert_eq!(out.unwrap_err(), Error::ProtocolError);
    }

    #[test]
    fn test_process_query_empty_array() {
        let input = RespVal::Array(Some(vec![]));
        let out = process_query(input);
        assert_eq!(out.unwrap_err(), Error::EmptyQuery);

        let input = RespVal::Array(None);
        let out = process_query(input);
        assert_eq!(out.unwrap_err(), Error::EmptyQuery);
    }
}
