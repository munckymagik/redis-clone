use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
    response_ext::ResponseExt,
};
use byte_glob;
use std::convert::TryInto;

pub(crate) fn del_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let mut count = 0;

    for key in request.bs_arguments() {
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

    for key in request.bs_arguments() {
        if db.contains_key(key) {
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
    let pattern = request.bs_arg(0)?;

    let results: Vec<_> = if pattern.as_ref() == b"*" {
        db.keys().collect()
    } else {
        db.keys().filter(|key| byte_glob::glob(pattern, key)).collect()
    };

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
    let key = request.bs_arg(0)?;

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
    match req.bs_maybe_arg(0) {
        Some(sub_command) => match sub_command.to_lowercase().as_ref() {
            b"help" => response.add_reply_help(req.command(), OBJECT_HELP),
            b"encoding" => {
                let key = match req.bs_maybe_arg(1) {
                    Some(k) => k,
                    None => {
                        response.add_reply_subcommand_syntax_error(req.command(), sub_command);
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
            _ => response.add_reply_subcommand_syntax_error(req.command(), sub_command),
        },
        None => {
            response.add_reply_subcommand_syntax_error(req.command(), "(none)");
        }
    }

    Ok(())
}
