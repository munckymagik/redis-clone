use crate::{
    db::Database, errors::Result, request::Request, response::Response, response_ext::ResponseExt,
};

mod command;
mod debug;
mod del;
mod exists;
mod flushdb;
mod get;
mod incr_decr;
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
        if !is_valid_arity(self.arity.into(), request.arity()) {
            response.add_reply_wrong_number_of_arguments(&request.command);
            return Ok(());
        }

        (self.handler)(db, request, response)
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
        name: "debug",
        handler: debug::call,
        arity: -2,
    },
    RedisCommand {
        name: "exists",
        handler: exists::call,
        arity: -2,
    },
    RedisCommand {
        name: "incr",
        handler: incr_decr::incr,
        arity: 2,
    },
    RedisCommand {
        name: "decr",
        handler: incr_decr::decr,
        arity: 2,
    },
    RedisCommand {
        name: "incrby",
        handler: incr_decr::incrby,
        arity: 3,
    },
    RedisCommand {
        name: "decrby",
        handler: incr_decr::decrby,
        arity: 3,
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
