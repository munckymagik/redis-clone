use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
};
use globber::Pattern;
use std::convert::TryInto;

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
    let pattern = request.arg(0)?;
    let matcher = match Pattern::new(pattern) {
        Ok(m) => m,
        Err(_) => {
            response.add_array_len(0);
            return Ok(());
        }
    };

    let results: Vec<_> = if pattern == "*" {
        db.keys().collect()
    } else {
        db.keys().filter(|key| matcher.matches(key)).collect()
    };

    response.add_array_len(results.len().try_into().unwrap());
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
                RObj::String(_) => "string",
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
