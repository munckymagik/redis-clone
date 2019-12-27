use crate::{
    errors::{Error, Result},
    request::Request,
    response::Response,
};

mod command;

type RedisCommandProc = fn(req: &Request) -> Result<Response>;

pub struct RedisCommand<'a> {
    pub name: &'a str,
    pub handler: RedisCommandProc,
    pub arity: i32,
}

impl RedisCommand<'_> {
    pub fn execute(&self, request: &Request) -> Result<Response> {
        (self.handler)(request)
    }
}

fn unimplemented_command(_req: &Request) -> Result<Response> {
    Err(Error::UnimplementedCommand)
}

static COMMAND_TABLE: &[RedisCommand] = &[
    RedisCommand {
        name: "get",
        handler: unimplemented_command,
        arity: 2,
    },
    RedisCommand {
        name: "set",
        handler: unimplemented_command,
        arity: -3,
    },
    RedisCommand {
        name: "del",
        handler: unimplemented_command,
        arity: -2,
    },
    RedisCommand {
        name: "command",
        handler: command::call,
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
