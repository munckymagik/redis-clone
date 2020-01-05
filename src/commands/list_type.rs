use crate::{db::Database, errors::Result, request::Request, response::Response};
use std::convert::TryInto;

pub(crate) fn llen(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let key = request.arg(0).unwrap();

    if let Some(value) = db.get(key) {
        response.add_integer(value.len().try_into().unwrap());
        return Ok(());
    }

    response.add_integer(0);

    Ok(())
}

pub(crate) fn lpush(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let key = request.arg(0).unwrap();
    let first_value = request.arg(1).unwrap();

    db.insert(key.to_string(), first_value.to_string());
    response.add_integer(1);

    Ok(())
}
