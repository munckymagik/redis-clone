use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn incr(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    incr_decr(db, request, response, 1)
}

pub(crate) fn incrby(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    if let Some(increment) = parse_i64_or_reply_with_error(response, &request.arg(1).unwrap()) {
        return incr_decr(db, request, response, increment);
    }

    Ok(())
}

pub(crate) fn decrby(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    if let Some(increment) = parse_i64_or_reply_with_error(response, &request.arg(1).unwrap()) {
        return incr_decr(db, request, response, -increment);
    }

    Ok(())
}

pub(crate) fn decr(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    incr_decr(db, request, response, -1)
}

fn incr_decr(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
    increment: i64,
) -> Result<()> {
    let key = request.arg(0).unwrap();

    if db.contains_key(key) {
        let old_value = &db[key];

        if let Some(value) = parse_i64_or_reply_with_error(response, old_value) {
            if let Some(new_value) = value.checked_add(increment) {
                db.insert(key.to_string(), new_value.to_string());
                response.add_integer(new_value);
            } else {
                response.add_error("ERR increment or decrement would overflow")
            }
        }
    } else {
        db.insert(key.to_string(), increment.to_string());
        response.add_integer(increment);
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
