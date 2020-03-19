use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
    response_ext::ResponseExt,
};
use byte_glob;
use std::{
    convert::TryInto,
    time::{Duration, Instant},
};

pub(crate) fn del_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let mut count = 0;

    for key in request.arguments() {
        if db.remove(key).is_some() {
            count += 1;
        }
    }

    response.add_integer(count);

    Ok(())
}

pub(crate) fn exists_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let mut count = 0;

    for key in request.arguments() {
        if db.get(key).is_some() {
            count += 1;
        }
    }

    response.add_integer(count);

    Ok(())
}

pub(crate) fn keys_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let pattern = request.arg(0)?;
    let all_keys = pattern.as_ref() == b"*";

    let results = db.filter_keys(|key| all_keys || byte_glob::glob(pattern, key));

    response.add_array_len(results.len().try_into()?);
    for key in results {
        response.add_bulk_string(key);
    }

    Ok(())
}

pub(crate) fn type_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(value) => {
            let type_name = match value {
                RObj::Int(_) | RObj::String(_) => "string",
                RObj::List(_) => "list",
            };

            response.add_simple_string(type_name);
        }
        None => {
            response.add_simple_string("none");
        }
    }

    Ok(())
}

const OBJECT_HELP: &[&str] = &[
    "ENCODING <key> -- Return the kind of internal representation used in order to store the value associated with a key.",
    "FREQ <key> -- NOT SUPPORTED",
    "IDLETIME <key> -- NOT SUPPORTED",
    "REFCOUNT <key> -- NOT SUPPORTED",
];

pub(crate) fn object_command(
    db: &mut Database,
    req: &Request,
    response: &mut Response,
) -> Result<()> {
    match req.maybe_arg(0) {
        Some(sub_command) => match sub_command.to_lowercase().as_ref() {
            b"help" => response.add_reply_help(req.command(), OBJECT_HELP),
            b"encoding" => {
                let key = match req.maybe_arg(1) {
                    Some(k) => k,
                    None => {
                        response.add_reply_subcommand_syntax_error(
                            req.command(),
                            sub_command.as_byte_str(),
                        );
                        return Ok(());
                    }
                };

                match db.get(key) {
                    Some(value) => {
                        let type_name = match value {
                            RObj::Int(_) => "int",
                            RObj::String(_) => "byte_string",
                            RObj::List(_) => "vecdeque",
                        };

                        response.add_bulk_string(type_name);
                    }
                    None => {
                        response.add_null_string();
                    }
                }
            }
            _ => {
                response.add_reply_subcommand_syntax_error(req.command(), sub_command.as_byte_str())
            }
        },
        None => {
            response.add_reply_subcommand_syntax_error(req.command(), "(none)".into());
        }
    }

    Ok(())
}

pub(crate) fn expire_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    if db.get(key).is_none() {
        response.add_integer(0);
        return Ok(());
    }

    let seconds: i64 = parse_arg_or_reply_with_err!(1, request, response);

    if !seconds.is_positive() {
        db.remove(key);
        response.add_integer(1);
        return Ok(());
    }

    let expires_at = Instant::now() + Duration::from_secs(seconds.try_into()?);
    let res = db.set_expire(key, expires_at);
    response.add_integer(res.into());

    Ok(())
}

pub(crate) fn persist_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    if db.get(key).is_none() {
        response.add_integer(0);
        return Ok(());
    }

    let res = db.persist(key);
    response.add_integer(res.into());
    Ok(())
}

pub(crate) fn ttl_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    if db.get(key).is_none() {
        response.add_integer(-2);
        return Ok(());
    }

    let now = Instant::now();

    match db.get_expire(key) {
        // Has an active future expiration time
        Some(when) if when > now => {
            let ttl: Duration = when - now;
            response.add_integer(ttl.as_secs().try_into()?);
        }
        // Key exists but has no expiry
        None => response.add_integer(-1),
        // Should not be a possible given we are calling get beforehand
        _ => unreachable!(),
    }

    Ok(())
}
