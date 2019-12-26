use super::Error;
use crate::protocol::RespBuilder;

mod command;

type RedisCommandProc = fn(args: &[String]) -> Result<RespBuilder, Error>;

pub struct RedisCommand<'a> {
    pub name: &'a str,
    pub proc: RedisCommandProc,
    pub arity: i32,
}

fn get_command(_args: &[String]) -> Result<RespBuilder, Error> {
    Err(Error::UnimplementedCommand)
}
fn set_command(_args: &[String]) -> Result<RespBuilder, Error> {
    Err(Error::UnimplementedCommand)
}
fn del_command(_args: &[String]) -> Result<RespBuilder, Error> {
    Err(Error::UnimplementedCommand)
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
        proc: command::call,
        arity: -1,
    },
];

pub fn lookup_command(name: &str) -> Option<&RedisCommand> {
    COMMAND_TABLE.iter().find(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_command() {
        assert!(lookup_command("get").is_some());
        assert!(lookup_command("xxx").is_none());
    }
}
