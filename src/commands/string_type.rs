use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
    response_ext::ResponseExt,
};
use byte_string::ByteString;

pub(crate) fn set_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let value = request.arg(1)?;
    let mut nx = false;
    let mut xx = false;

    for arg in &request.arguments()[2..] {
        match arg.to_lowercase().as_ref() {
            "nx" if !xx => nx = true,
            "xx" if !nx => xx = true,
            _ => {
                response.add_error("ERR syntax error");
                return Ok(());
            }
        }
    }

    if nx && db.contains_key(key) || xx && !db.contains_key(key) {
        response.add_null_string();
        return Ok(());
    }

    db.insert(key.to_owned(), ByteString::from(value).into());
    response.add_simple_string("OK");

    Ok(())
}

pub(crate) fn get_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(RObj::BString(value)) => {
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
    let arg = ByteString::from(request.arg(1)?);

    if let Some(increment) = parse_i64_or_reply_with_error(response, &arg) {
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
        Some(RObj::BString(old_value)) => {
            if let Some(value) = parse_i64_or_reply_with_error(response, old_value) {
                if let Some(new_value) = value.checked_add(increment) {
                    db.insert(key.to_string(), ByteString::from(new_value.to_string()).into());
                    response.add_integer(new_value);
                } else {
                    response.add_error("ERR increment or decrement would overflow")
                }
            }
        }
        Some(RObj::Int(old_value)) => {
            if let Some(new_value) = old_value.checked_add(increment) {
                db.insert(key.to_string(), new_value.into());
                response.add_integer(new_value);
            } else {
                response.add_error("ERR increment or decrement would overflow")
            }
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            db.insert(key.to_string(), increment.into());
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
