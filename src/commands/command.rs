use super::COMMAND_TABLE;
use crate::{db::Database, errors::Result, request::Request, response_ext::ResponseExt, response::Response};
use std::convert::TryInto;

const COMMAND_HELP: &[&str] = &[
    "(no subcommand) -- Return details about all Redis commands.",
    "COUNT -- Return the total number of commands in this Redis server.",
    "GETKEYS <full-command> -- Return the keys from a full Redis command.",
    "INFO [command-name ...] -- Return details about multiple Redis commands.",
];

pub(crate) fn call(_: &mut Database, req: &Request, reply: &mut Response) -> Result<()> {
    match req.arg(0) {
        Some(sub_command) => match sub_command.as_ref() {
            "help" => {
                reply.add_reply_help(&req.command, COMMAND_HELP);
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

    Ok(())
}
