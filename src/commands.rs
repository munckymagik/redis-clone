use super::Error;
use crate::protocol::RespBuilder;
use std::convert::TryInto;

type RedisCommandProc = fn(args: &[String]) -> Result<RespBuilder, Error>;

pub struct RedisCommand<'a> {
    pub name: &'a str,
    pub proc: RedisCommandProc,
    pub arity: i32,
}

static COMMAND_TABLE: &[RedisCommand] = &[
    RedisCommand {
        name: "get",
        proc: get_command,
        arity: 2,
    },
    RedisCommand {
        name: "set",
        proc: set_command,
        arity: -3,
    },
    RedisCommand {
        name: "del",
        proc: del_command,
        arity: -2,
    },
    RedisCommand {
        name: "command",
        proc: command_command,
        arity: -1,
    },
];

fn get_command(_args: &[String]) -> Result<RespBuilder, Error> {
    Err(Error::UnimplementedCommand)
}
fn set_command(_args: &[String]) -> Result<RespBuilder, Error> {
    Err(Error::UnimplementedCommand)
}
fn del_command(_args: &[String]) -> Result<RespBuilder, Error> {
    Err(Error::UnimplementedCommand)
}

const COMMAND_HELP: &[&str] = &[
    "(no subcommand) -- Return details about all Redis commands.",
    "COUNT -- Return the total number of commands in this Redis server.",
    "GETKEYS <full-command> -- Return the keys from a full Redis command.",
    "INFO [command-name ...] -- Return details about multiple Redis commands.",
];

fn command_command(args: &[String]) -> Result<RespBuilder, Error> {
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

pub fn lookup_command(name: &str) -> Option<&RedisCommand> {
    COMMAND_TABLE.iter().find(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::RespVal;

    #[test]
    fn test_lookup_command() {
        assert!(lookup_command("get").is_some());
        assert!(lookup_command("xxx").is_none());
    }

    #[test]
    fn test_command_command() {
        // help
        let output = command_command(&["help".into()]).unwrap().decode().unwrap();
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
        let output = command_command(&["count".into()])
            .unwrap()
            .decode()
            .unwrap();
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
        // let output = command_command(&["info".into(), "get".into()]).unwrap();
        // assert_eq!(output, expected_output);

        // no arg default
        let output = command_command(&[]).unwrap().decode().unwrap();
        if let RespVal::Array(Some(ref replies)) = output {
            assert_eq!(replies.len(), COMMAND_HELP.len());
        } else {
            panic!("not an array: {:?}", output);
        }

        // Unknown sub-command
        let output = command_command(&["xyz".into()]).unwrap().decode().unwrap();
        let expected =
            "ERR Unknown subcommand or wrong number of arguments for 'xyz'. Try COMMAND HELP.";
        if let RespVal::Error(message) = output {
            assert_eq!(message, expected);
        } else {
            panic!("not an error: {:?}", output);
        }
    }
}
