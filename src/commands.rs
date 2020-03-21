use crate::{
    db::Database, errors::Result, request::Request, response::Response, response_ext::ResponseExt,
};
use byte_string::ByteStr;

mod hash_type;
mod keyspace;
mod list_type;
mod server;
mod string_type;

type RedisCommandProc = fn(db: &mut Database, req: &Request, resp: &mut Response) -> Result<()>;

pub struct RedisCommand<'a> {
    pub name: &'a [u8],
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
            response.add_reply_wrong_number_of_arguments(request.command());
            return Ok(());
        }

        (self.handler)(db, request, response)
    }
}

fn is_valid_arity(arity: i64, given: i64) -> bool {
    assert!(arity != 0);
    arity == given || (arity < 0 && given >= arity.abs())
}

static COMMAND_TABLE: &[RedisCommand] = &[
    RedisCommand {
        name: b"get",
        handler: string_type::get_command,
        arity: 2,
    },
    RedisCommand {
        name: b"set",
        handler: string_type::set_command,
        arity: -3,
    },
    RedisCommand {
        name: b"del",
        handler: keyspace::del_command,
        arity: -2,
    },
    RedisCommand {
        name: b"exists",
        handler: keyspace::exists_command,
        arity: -2,
    },
    RedisCommand {
        name: b"expire",
        handler: keyspace::expire_command,
        arity: 3,
    },
    RedisCommand {
        name: b"persist",
        handler: keyspace::persist_command,
        arity: 2,
    },
    RedisCommand {
        name: b"ttl",
        handler: keyspace::ttl_command,
        arity: 2,
    },
    RedisCommand {
        name: b"incr",
        handler: string_type::incr_command,
        arity: 2,
    },
    RedisCommand {
        name: b"decr",
        handler: string_type::decr_command,
        arity: 2,
    },
    RedisCommand {
        name: b"incrby",
        handler: string_type::incrby_command,
        arity: 3,
    },
    RedisCommand {
        name: b"decrby",
        handler: string_type::decrby_command,
        arity: 3,
    },
    RedisCommand {
        name: b"rpush",
        handler: list_type::rpush_command,
        arity: -3,
    },
    RedisCommand {
        name: b"lpush",
        handler: list_type::lpush_command,
        arity: -3,
    },
    RedisCommand {
        name: b"linsert",
        handler: list_type::linsert_command,
        arity: 5,
    },
    RedisCommand {
        name: b"rpop",
        handler: list_type::rpop_command,
        arity: 2,
    },
    RedisCommand {
        name: b"lpop",
        handler: list_type::lpop_command,
        arity: 2,
    },
    RedisCommand {
        name: b"llen",
        handler: list_type::llen_command,
        arity: 2,
    },
    RedisCommand {
        name: b"lindex",
        handler: list_type::lindex_command,
        arity: 3,
    },
    RedisCommand {
        name: b"lset",
        handler: list_type::lset_command,
        arity: 4,
    },
    RedisCommand {
        name: b"lrange",
        handler: list_type::lrange_command,
        arity: 4,
    },
    RedisCommand {
        name: b"ltrim",
        handler: list_type::ltrim_command,
        arity: 4,
    },
    RedisCommand {
        name: b"lrem",
        handler: list_type::lrem_command,
        arity: 4,
    },
    RedisCommand {
        name: b"hset",
        handler: hash_type::hset_command,
        arity: -4,
    },
    RedisCommand {
        name: b"hget",
        handler: hash_type::hget_command,
        arity: 3,
    },
    RedisCommand {
        name: b"hmset",
        handler: hash_type::hmset_command,
        arity: -4,
    },
    RedisCommand {
        name: b"hmget",
        handler: hash_type::hmget_command,
        arity: -3,
    },
    RedisCommand {
        name: b"hgetall",
        handler: hash_type::hgetall_command,
        arity: 2,
    },
    RedisCommand {
        name: b"command",
        handler: server::command_command,
        arity: -1,
    },
    RedisCommand {
        name: b"debug",
        handler: server::debug_command,
        arity: -2,
    },
    RedisCommand {
        name: b"flushdb",
        handler: server::flushdb_command,
        arity: -1,
    },
    RedisCommand {
        name: b"keys",
        handler: keyspace::keys_command,
        arity: 2,
    },
    RedisCommand {
        name: b"type",
        handler: keyspace::type_command,
        arity: 2,
    },
    RedisCommand {
        name: b"object",
        handler: keyspace::object_command,
        arity: -2,
    },
];

pub fn lookup(name: ByteStr) -> Option<&RedisCommand> {
    COMMAND_TABLE
        .iter()
        .find(|c| name.eq_ignore_ascii_case(c.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup() {
        assert!(lookup("get".into()).is_some());
        assert!(lookup("GET".into()).is_some());
        assert!(lookup("xxx".into()).is_none());
    }

    #[test]
    fn test_is_valid_arity() {
        assert!(!is_valid_arity(2, 0));
        assert!(!is_valid_arity(2, 1));
        assert!(is_valid_arity(2, 2));
        assert!(!is_valid_arity(2, 3));

        assert!(!is_valid_arity(-2, 0));
        assert!(!is_valid_arity(-2, 1));
        assert!(is_valid_arity(-2, 2));
        assert!(is_valid_arity(-2, 3));
    }

    #[test]
    #[should_panic]
    fn test_is_valid_arity_panics_on_zero() {
        is_valid_arity(0, 0);
    }
}
