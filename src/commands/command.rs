use std::convert::TryInto;

use crate::protocol::RespBuilder;
use crate::Result;

use super::COMMAND_TABLE;

pub(crate) fn call(args: &[String]) -> Result<RespBuilder> {
    let mut reply = RespBuilder::new();

    match args.get(0) {
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
        let output = call(&["help".into()]).unwrap().decode().unwrap();
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
        let output = call(&["count".into()]).unwrap().decode().unwrap();
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
        let output = call(&[]).unwrap().decode().unwrap();
        if let RespVal::Array(Some(ref replies)) = output {
            assert_eq!(replies.len(), COMMAND_HELP.len());
        } else {
            panic!("not an array: {:?}", output);
        }

        // Unknown sub-command
        let output = call(&["xyz".into()]).unwrap().decode().unwrap();
        let expected =
            "ERR Unknown subcommand or wrong number of arguments for 'xyz'. Try COMMAND HELP.";
        if let RespVal::Error(message) = output {
            assert_eq!(message, expected);
        } else {
            panic!("not an error: {:?}", output);
        }
    }
}
