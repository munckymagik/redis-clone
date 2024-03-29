use super::{RedisCommand, COMMAND_TABLE};
use crate::{
    db::Database, errors::Error, errors::Result, request::Request, response::Response,
    response_ext::ResponseExt,
};
use std::convert::TryInto;

const COMMAND_HELP: &[&str] = &[
    "(no subcommand) -- Return details about all Redis commands.",
    "COUNT -- Return the total number of commands in this Redis server.",
    "GETKEYS -- NOT SUPPORTED.",
    "INFO [command-name ...] -- Return details about multiple Redis commands.",
];

pub(crate) fn command_command(_: &mut Database, req: &Request, reply: &mut Response) -> Result<()> {
    match req.maybe_arg(0) {
        Some(sub_command) => match sub_command.to_lowercase().as_ref() {
            b"help" => reply.add_reply_help(req.command(), COMMAND_HELP),
            b"count" => reply.add_integer(COMMAND_TABLE.len().try_into()?),
            b"info" => {
                let requested = &req.arguments()[1..];
                reply.add_array_len(requested.len().try_into()?);
                for cmd in requested {
                    match super::lookup(cmd.as_byte_str()) {
                        Some(cmd) => command_reply(reply, cmd),
                        None => reply.add_null_array(),
                    }
                }
            }
            _ => reply.add_reply_subcommand_syntax_error(req.command(), sub_command.as_byte_str()),
        },
        None => {
            reply.add_array_len(COMMAND_TABLE.len().try_into()?);

            for cmd in COMMAND_TABLE {
                command_reply(reply, cmd);
            }
        }
    }

    Ok(())
}

fn command_reply(reply: &mut Response, cmd: &RedisCommand) {
    reply.add_array_len(2);
    reply.add_bulk_string(cmd.name);
    reply.add_integer(cmd.arity.into());
}

const DEBUG_HELP: &[&str] = &[
    "PANIC -- Crash the server simulating a panic.",
    "ERROR -- Simulate an error.",
];

pub(crate) fn debug_command(_: &mut Database, req: &Request, reply: &mut Response) -> Result<()> {
    let sub_command = req.arg(0)?.to_lowercase();

    match sub_command.as_ref() {
        b"help" => reply.add_reply_help(req.command(), DEBUG_HELP),
        b"panic" => panic!("A deliberate panic from DEBUG PANIC"),
        b"error" => {
            return Err(Error::from("A deliberate error from DEBUG ERROR"));
        }
        _ => reply.add_reply_subcommand_syntax_error(req.command(), sub_command.as_byte_str()),
    };

    Ok(())
}

pub(crate) fn flushdb_command(
    db: &mut Database,
    _request: &Request,
    response: &mut Response,
) -> Result<()> {
    db.clear();

    response.add_simple_string("OK");

    Ok(())
}
