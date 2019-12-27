use crate::{errors::Error, protocol::RespBuilder, request::Request};

mod command;

type RedisCommandProc = fn(req: &Request) -> Result<RespBuilder, Error>;

pub struct RedisCommand<'a> {
    pub name: &'a str,
    pub proc: RedisCommandProc,
    pub arity: i32,
}

fn unimplemented_command(_req: &Request) -> Result<RespBuilder, Error> {
    Err(Error::UnimplementedCommand)
}

static COMMAND_TABLE: &[RedisCommand] = &[
    RedisCommand {
        name: "get",
        proc: unimplemented_command,
        arity: 2,
    },
    RedisCommand {
        name: "set",
        proc: unimplemented_command,
        arity: -3,
    },
    RedisCommand {
        name: "del",
        proc: unimplemented_command,
        arity: -2,
    },
    RedisCommand {
        name: "command",
        proc: command::call,
        arity: -1,
    },
];

pub fn lookup(name: &str) -> Option<&RedisCommand> {
    COMMAND_TABLE.iter().find(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup() {
        assert!(lookup("get").is_some());
        assert!(lookup("xxx").is_none());
    }
}
