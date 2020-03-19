use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
    response_ext::ResponseExt,
};
use byte_string::ByteString;
use std::{collections::HashMap, convert::TryInto};

pub(crate) fn hset_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let values = &request.arguments()[1..];

    if values.len() % 2 != 0 {
        // Note: HSET and HMSET are the handled by the same function in Redis
        // and the error seems to assume the command is HMSET
        response.add_error("ERR wrong number of arguments for HMSET");
        return Ok(());
    }

    match db.get_mut(key) {
        Some(RObj::Hash(ref mut hash)) => {
            let prev_len = hash.len();
            let new_hash = values
                .chunks(2)
                .map(|pair| (pair[0].clone(), pair[1].clone()));
            hash.extend(new_hash);

            let count_keys_added = hash.len() - prev_len;
            response.add_integer(count_keys_added.try_into()?);
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            let new_hash = values
                .chunks(2)
                .map(|pair| (pair[0].clone(), pair[1].clone()))
                .collect::<HashMap<ByteString, ByteString>>();
            let count_keys_added = new_hash.len();
            db.insert(key.clone(), RObj::Hash(new_hash));

            response.add_integer(count_keys_added.try_into()?);
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
    let hash_key = request.arg(1)?;

    match db.get(key) {
        Some(RObj::Hash(ref hash)) => match hash.get(hash_key) {
            Some(value) => response.add_bulk_string(value),
            _ => response.add_null_string(),
        },
        Some(_) => response.add_reply_wrong_type(),
        None => {
            response.add_null_string();
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
