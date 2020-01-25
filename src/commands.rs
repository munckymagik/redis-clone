use crate::{
    db::Database, errors::Result, request::Request, response::Response, response_ext::ResponseExt,
};

mod keyspace;
mod list_type;
mod server;
mod string_type;

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
        handler: string_type::get,
        arity: 2,
    },
    RedisCommand {
        name: "set",
        handler: string_type::set,
        arity: -3,
    },
    RedisCommand {
        name: "del",
        handler: keyspace::del,
        arity: -2,
    },
    RedisCommand {
        name: "exists",
        handler: keyspace::exists,
        arity: -2,
    },
    RedisCommand {
        name: "incr",
        handler: string_type::incr,
        arity: 2,
    },
    RedisCommand {
        name: "decr",
        handler: string_type::decr,
        arity: 2,
    },
    RedisCommand {
        name: "incrby",
        handler: string_type::incrby,
        arity: 3,
    },
    RedisCommand {
        name: "decrby",
        handler: string_type::decrby,
        arity: 3,
    },
    RedisCommand {
        name: "rpush",
        handler: list_type::rpush,
        arity: -3,
    },
    RedisCommand {
        name: "lpush",
        handler: list_type::lpush,
        arity: -3,
    },
    RedisCommand {
        name: "linsert",
        handler: list_type::linsert,
        arity: 5,
    },
    RedisCommand {
        name: "rpop",
        handler: list_type::rpop,
        arity: 2,
    },
    RedisCommand {
        name: "lpop",
        handler: list_type::lpop,
        arity: 2,
    },
    RedisCommand {
        name: "llen",
        handler: list_type::llen,
        arity: 2,
    },
    RedisCommand {
        name: "lindex",
        handler: list_type::lindex,
        arity: 3,
    },
    RedisCommand {
        name: "lset",
        handler: list_type::lset,
        arity: 4,
    },
    RedisCommand {
        name: "lrange",
        handler: list_type::lrange,
        arity: 4,
    },
    RedisCommand {
        name: "ltrim",
        handler: list_type::ltrim,
        arity: 4,
    },
    RedisCommand {
        name: "lrem",
        handler: list_type::lrem,
        arity: 4,
    },
    RedisCommand {
        name: "command",
        handler: server::command,
        arity: -1,
    },
    RedisCommand {
        name: "debug",
        handler: server::debug,
        arity: -2,
    },
    RedisCommand {
        name: "flushdb",
        handler: server::flushdb,
        arity: -1,
    },
    RedisCommand {
        name: "keys",
        handler: keyspace::keys,
        arity: 2,
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
