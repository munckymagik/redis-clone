use crate::{db::Database, errors::Result, request::Request, response::Response};

mod command;
mod del;
mod get;
mod set;

type RedisCommandProc = fn(db: &mut Database, req: &Request, resp: &mut Response) -> Result<()>;

pub struct RedisCommand<'a> {
    pub name: &'a str,
    pub handler: RedisCommandProc,
    pub arity: i32,
}

impl RedisCommand<'_> {
    pub fn execute(
        &self,
        db: &mut Database,
        request: &Request,
        response: &mut Response,
    ) -> Result<()> {
        (self.handler)(db, request, response)
    }
}

static COMMAND_TABLE: &[RedisCommand] = &[
    RedisCommand {
        name: "get",
        handler: get::call,
        arity: 2,
    },
    RedisCommand {
        name: "set",
        handler: set::call,
        arity: -3,
    },
    RedisCommand {
        name: "del",
        handler: del::call,
        arity: -2,
    },
    RedisCommand {
        name: "command",
        handler: command::call,
        arity: -1,
    },
];

pub fn lookup(name: &str) -> Option<&RedisCommand> {
    let name = name.to_lowercase();
    COMMAND_TABLE.iter().find(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup() {
        assert!(lookup("get").is_some());
        assert!(lookup("GET").is_some());
        assert!(lookup("xxx").is_none());
    }
}
