use super::COMMAND_TABLE;
use crate::{db::Database, errors::Result, request::Request, response_ext::ResponseExt, response::Response};
use std::convert::TryInto;

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

const COMMAND_HELP: &[&str] = &[
    "(no subcommand) -- Return details about all Redis commands.",
    "COUNT -- Return the total number of commands in this Redis server.",
    "GETKEYS <full-command> -- Return the keys from a full Redis command.",
    "INFO [command-name ...] -- Return details about multiple Redis commands.",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Database, Response) {
        (Database::new(), Response::new())
    }

    fn perform(request: &Request) -> String {
        let (mut db, mut response) = setup();
        call(&mut db, &request, &mut response).unwrap();
        let output = &response.as_bytes();
        std::str::from_utf8(output).unwrap().to_string()
    }

    #[test]
    fn help() {
        let request = Request::new("command", &["help"]);
        let output = perform(&request);
        assert_eq!(&output[..12], "*5\r\n+COMMAND");
    }

    #[test]
    fn count() {
        let request = Request::new("command", &["count"]);
        let output = perform(&request);
        let expected = format!(":{}\r\n", COMMAND_TABLE.len());
        assert_eq!(output, expected);
    }

    #[test]
    fn default() {
        let request = Request::new("command", &[]);
        let output = perform(&request);

        let expected_prefix = format!("*{}\r\n", COMMAND_TABLE.len());
        assert_eq!(&output[..4], expected_prefix);
    }

    #[test]
    fn unknown_subcommand() {
        let request = Request::new("command", &["xyz"]);
        let output = perform(&request);

        let expected_prefix = "-ERR Unknown subcommand";
        assert_eq!(&output[..23], expected_prefix);
    }
}
