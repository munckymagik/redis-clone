use std::convert::TryInto;

use crate::{errors::Result, request::Request, response::Response};

use super::COMMAND_TABLE;

pub(crate) fn call(req: &Request) -> Result<Response> {
    let mut reply = Response::new();

    match req.arg(0) {
        Some(sub_command) => match sub_command.as_ref() {
            "help" => {
                reply.add_array_len(COMMAND_HELP.len().try_into().unwrap());
                for line in COMMAND_HELP {
                    reply.add_simple_string(line);
                }
            }
            "count" => reply.add_integer(COMMAND_TABLE.len().try_into().unwrap()),
            _ => {
                let msg = format!(
                    "ERR Unknown subcommand or wrong number of arguments for '{}'. Try COMMAND HELP.",
                    sub_command,
                );
                reply.add_error(&msg);
            }
        },
        None => {
            reply.add_array_len(COMMAND_TABLE.len().try_into().unwrap());

            for cmd in COMMAND_TABLE {
                reply.add_array_len(2);
                reply.add_bulk_string(cmd.name);
                reply.add_integer(cmd.arity.into());
            }
        }
    }

    Ok(reply)
}

const COMMAND_HELP: &[&str] = &[
    "(no subcommand) -- Return details about all Redis commands.",
    "COUNT -- Return the total number of commands in this Redis server.",
    "GETKEYS <full-command> -- Return the keys from a full Redis command.",
    "INFO [command-name ...] -- Return details about multiple Redis commands.",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::RespVal;

    #[test]
    fn test_call() {
        // help
        let request = Request::new("command", &["help"]);
        let output = call(&request).unwrap().decode().unwrap();
        if let RespVal::Array(Some(ref lines)) = output {
            assert_eq!(lines.len(), COMMAND_HELP.len());
            assert_eq!(
                lines.get(0).unwrap(),
                &RespVal::SimpleString(
                    "(no subcommand) -- Return details about all Redis commands.".into()
                )
            );
        } else {
            panic!("not an array: {:?}", output);
        }

        // count
        let request = Request::new("command", &["count"]);
        let output = call(&request).unwrap().decode().unwrap();
        assert_eq!(
            output,
            RespVal::Integer(COMMAND_TABLE.len().try_into().unwrap())
        );

        // info
        // let expected_output = RespVal::Array(Some(vec![
        //     RespVal::Array(Some(vec![
        //         RespVal::BulkString(Some("set".into())),
        //         RespVal::Integer("-3"),
        //         RespVal::Array(Some(vec![
        //             RespVal::SimpleString("write"),
        //             RespVal::SimpleString("denyoom"),
        //         ])),
        //         RespVal::Integer("1"),
        //         RespVal::Integer("1"),
        //         RespVal::Integer("1"),
        //     ]),
        // ]);
        // let output = call(&["info".into(), "get".into()]).unwrap();
        // assert_eq!(output, expected_output);

        // no arg default
        let request = Request::new("command", &[]);
        let output = call(&request).unwrap().decode().unwrap();
        if let RespVal::Array(Some(ref replies)) = output {
            assert_eq!(replies.len(), COMMAND_HELP.len());
        } else {
            panic!("not an array: {:?}", output);
        }

        // Unknown sub-command
        let request = Request::new("command", &["xyz"]);
        let output = call(&request).unwrap().decode().unwrap();
        let expected =
            "ERR Unknown subcommand or wrong number of arguments for 'xyz'. Try COMMAND HELP.";
        if let RespVal::Error(message) = output {
            assert_eq!(message, expected);
        } else {
            panic!("not an error: {:?}", output);
        }
    }
}
