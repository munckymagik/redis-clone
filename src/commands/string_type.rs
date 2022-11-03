use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
    response_ext::ResponseExt,
};
use byte_string::ByteString;
use std::{
    convert::{TryFrom, TryInto},
    time::{Duration, Instant},
};

pub(crate) fn set_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let value = request.arg(1)?;
    let mut nx = false;
    let mut xx = false;
    let mut maybe_ttl: Option<i64> = None;
    let mut args = &request.arguments()[2..];

    while let Some(arg) = args.get(0) {
        let arg = arg.to_lowercase();
        match arg.as_ref() {
            b"nx" if !xx => nx = true,
            b"xx" if !nx => xx = true,
            b"ex" | b"px" if maybe_ttl.is_none() && args.get(1).is_some() => {
                let mut ttl: i64 = parse_or_reply_with_err!(args[1], response);
                if !ttl.is_positive() {
                    response.add_error("ERR invalid expire time in set");
                    return Ok(());
                }
                if arg[0] == b'e' {
                    ttl *= 1000;
                }
                maybe_ttl = Some(ttl);
                args = &args[1..];
            }
            _ => {
                response.add_error("ERR syntax error");
                return Ok(());
            }
        }

        args = &args[1..];
    }

    let is_existing = db.get(key).is_some();
    if nx && is_existing || xx && !is_existing {
        response.add_null_string();
    } else {
        db.insert(key.clone(), value.clone().into());
        response.add_simple_string("OK");
    }

    if let Some(millis) = maybe_ttl {
        let expires_at = Instant::now() + Duration::from_millis(millis.try_into()?);
        db.set_expire(key, expires_at);
    }

    Ok(())
}

pub(crate) fn get_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(RObj::String(value)) => {
            response.add_bulk_string(value);
        }
        Some(RObj::Int(value)) => {
            response.add_bulk_string(&value.to_string());
        }
        Some(_) => response.add_reply_wrong_type(),
        None => response.add_null_string(),
    }

    Ok(())
}

pub(crate) fn mset_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let arguments = request.arguments();

    if arguments.len() % 2 != 0 {
        response.add_error("ERR wrong number of arguments for MSET");
        return Ok(());
    }

    let pairs = arguments.chunks(2).flat_map(<&[ByteString; 2]>::try_from);

    for [key, value] in pairs {
        db.insert(key.clone(), value.clone().into());
    }

    response.add_simple_string("OK");

    Ok(())
}

pub(crate) fn mget_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let keys = request.arguments();

    let len: i64 = keys.len().try_into()?;
    response.add_array_len(len);

    for key in keys {
        match db.get(key) {
            Some(RObj::Int(value)) => response.add_bulk_string(value.to_string()),
            Some(RObj::String(value)) => response.add_bulk_string(value),
            _ => response.add_null_string(),
        };
    }

    Ok(())
}

pub(crate) fn incr_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    general_incr(db, request, response, 1)
}

pub(crate) fn decr_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    general_incr(db, request, response, -1)
}

pub(crate) fn incrby_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let arg = ByteString::from(request.arg(1)?);

    if let Some(increment) = parse_i64_or_reply_with_error(response, &arg) {
        return general_incr(db, request, response, increment);
    }

    Ok(())
}

pub(crate) fn decrby_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let arg = request.arg(1)?;

    if let Some(increment) = parse_i64_or_reply_with_error(response, arg) {
        return general_incr(db, request, response, -increment);
    }

    Ok(())
}

fn general_incr(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
    increment: i64,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(RObj::Int(old_value)) => {
            if let Some(new_value) = old_value.checked_add(increment) {
                db.insert(key.to_owned(), new_value.into());
                response.add_integer(new_value);
            } else {
                response.add_error("ERR increment or decrement would overflow")
            }
        }
        // If RObj could not parse the existing value as an int when it was set
        // there is no point us trying now. So reply with NaN.
        Some(RObj::String(_)) => response.add_reply_not_a_number(),
        Some(_) => response.add_reply_wrong_type(),
        None => {
            db.insert(key.to_owned(), increment.into());
            response.add_integer(increment);
        }
    }

    Ok(())
}

fn parse_i64_or_reply_with_error(response: &mut Response, value: &ByteString) -> Option<i64> {
    match value.parse() {
        Ok(v) => Some(v),
        Err(_) => {
            response.add_reply_not_a_number();
            None
        }
    }
}
