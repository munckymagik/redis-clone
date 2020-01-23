use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
    response_ext::ResponseExt,
};

pub(crate) fn incr(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    incr_decr(db, request, response, 1)
}

pub(crate) fn decr(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    incr_decr(db, request, response, -1)
}

pub(crate) fn incrby(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let arg = request.arg(1)?;

    if let Some(increment) = parse_i64_or_reply_with_error(response, &arg) {
        return incr_decr(db, request, response, increment);
    }

    Ok(())
}

pub(crate) fn decrby(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let arg = request.arg(1)?;

    if let Some(increment) = parse_i64_or_reply_with_error(response, &arg) {
        return incr_decr(db, request, response, -increment);
    }

    Ok(())
}

fn incr_decr(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
    increment: i64,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(RObj::String(old_value)) => {
            if let Some(value) = parse_i64_or_reply_with_error(response, old_value) {
                if let Some(new_value) = value.checked_add(increment) {
                    db.insert(key.to_string(), new_value.to_string().into());
                    response.add_integer(new_value);
                } else {
                    response.add_error("ERR increment or decrement would overflow")
                }
            }
        },
        Some(_) => response.add_reply_wrong_type(),
        None => {
            db.insert(key.to_string(), increment.to_string().into());
            response.add_integer(increment);
        },
    }

    Ok(())
}

fn parse_i64_or_reply_with_error(response: &mut Response, value: &str) -> Option<i64> {
    match value.parse() {
        Ok(v) => Some(v),
        Err(_) => {
            response.add_error("ERR value is not an integer or out of range");
            None
        }
    }
}
