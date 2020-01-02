use crate::{db::Database, errors::Error, errors::Result, request::Request, response::Response};

mod command;
mod del;
mod flushdb;
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
        self.validate_arity(request)?;

        (self.handler)(db, request, response)
    }

    fn validate_arity(&self, request: &Request) -> Result<()> {
        if is_valid_arity(self.arity.into(), request.arity()) {
            Ok(())
        } else {
            Err(Error::MissingArguments(request.command.to_owned()))
        }
    }
}

fn is_valid_arity(arity: i64, given: i64) -> bool {
    arity == given || (arity < 0 && given >= arity.abs())
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
    RedisCommand {
        name: "flushdb",
        handler: flushdb::call,
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

    #[test]
    fn test_is_valid_arity() {
        assert!(is_valid_arity(2, 2));
        assert!(!is_valid_arity(2, 1));
        assert!(!is_valid_arity(2, 3));

        assert!(is_valid_arity(-2, 2));
        assert!(!is_valid_arity(-2, 1));
        assert!(is_valid_arity(-2, 3));
    }
}
