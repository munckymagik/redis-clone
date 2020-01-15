use crate::{
    db::Database, errors::Error, errors::Result, request::Request, response::Response,
    response_ext::ResponseExt,
};

const COMMAND_HELP: &[&str] = &[
    "PANIC -- Crash the server simulating a panic.",
    "ERROR -- Simulate an error.",
];

pub(crate) fn call(_: &mut Database, req: &Request, reply: &mut Response) -> Result<()> {
    let sub_command = req.arg(0)?.to_lowercase();

    match sub_command.as_ref() {
        "help" => reply.add_reply_help(&req.command, COMMAND_HELP),
        "panic" => panic!("A deliberate panic from DEBUG PANIC"),
        "error" => {
            return Err(Error::from("A deliberate error from DEBUG ERROR"));
        }
        _ => reply.add_reply_subcommand_syntax_error(&req.command, &sub_command),
    };

    Ok(())
}
