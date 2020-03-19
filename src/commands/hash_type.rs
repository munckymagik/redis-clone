use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
    response_ext::ResponseExt,
};

pub(crate) fn hset_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let _values = &request.arguments()[1..];

    match db.get_mut(key) {
        Some(RObj::Hash(ref mut _hash)) => {
            response.add_integer(0);
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            response.add_integer(0);
        }
    }

    Ok(())
}

pub(crate) fn hmset_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let _values = &request.arguments()[1..];

    match db.get_mut(key) {
        Some(RObj::Hash(ref mut _hash)) => {
            response.add_integer(0);
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            response.add_integer(0);
        }
    }

    Ok(())
}

pub(crate) fn hget_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let _values = &request.arguments()[1..];

    match db.get(key) {
        Some(RObj::Hash(ref _hash)) => {
            response.add_integer(0);
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            response.add_integer(0);
        }
    }

    Ok(())
}

pub(crate) fn hmget_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let _values = &request.arguments()[1..];

    match db.get(key) {
        Some(RObj::Hash(ref _hash)) => {
            response.add_integer(0);
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            response.add_integer(0);
        }
    }

    Ok(())
}

pub(crate) fn hgetall_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let _values = &request.arguments()[1..];

    match db.get(key) {
        Some(RObj::Hash(ref _hash)) => {
            response.add_integer(0);
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            response.add_integer(0);
        }
    }

    Ok(())
}
