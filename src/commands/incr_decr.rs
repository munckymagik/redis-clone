use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn incr(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    incr_decr(db, request, response, 1)
}

pub(crate) fn incrby(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let increment = match request.arg(1).unwrap().parse() {
        Ok(n) => n,
        Err(_) => {
            response.add_error("ERR value is not an integer or out of range");
            return Ok(());
        }
    };

    incr_decr(db, request, response, increment)
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
        let value = &db[key];

        let value: i64 = match value.parse() {
            Ok(v) => v,
            Err(_) => {
                response.add_error("ERR value is not an integer or out of range");
                return Ok(());
            }
        };

        if let Some(value) = value.checked_add(increment) {
            db.insert(key.to_string(), value.to_string());
            response.add_integer(value);
        } else {
            response.add_error("ERR increment or decrement would overflow")
        }
    } else {
        db.insert(key.to_string(), increment.to_string());
        response.add_integer(increment);
    }

    Ok(())
}
