use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
    response_ext::ResponseExt,
};
use byte_string::ByteString;
use std::{collections::HashSet, convert::TryInto, iter::FromIterator};

pub(crate) fn sadd_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let values = &request.arguments()[1..];

    match db.get_mut(key) {
        Some(RObj::Set(ref mut set)) => {
            let prev_len = set.len();
            set.extend(values.into_iter().cloned());

            let count_values_added = set.len() - prev_len;
            response.add_integer(count_values_added.try_into()?);
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            let new_set = HashSet::from_iter(values.into_iter().cloned());
            let count_values_added = new_set.len();
            db.insert(key.clone(), RObj::Set(new_set));
            response.add_integer(count_values_added.try_into()?);
        }
    }

    Ok(())
}

pub(crate) fn scard_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(RObj::Set(value)) => response.add_integer(value.len().try_into()?),
        Some(_) => response.add_reply_wrong_type(),
        None => response.add_integer(0),
    }

    Ok(())
}

pub(crate) fn smembers_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(RObj::Set(ref set)) => {
            let len: i64 = set.len().try_into()?;
            response.add_array_len(len);

            for value in set {
                response.add_bulk_string(value);
            }
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            response.add_array_len(0);
        }
    }

    Ok(())
}

pub(crate) fn srem_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let values = &request.arguments()[1..];

    match db.get_mut(key) {
        Some(RObj::Set(ref mut set)) => {
            let prev_len = set.len();

            for value in values {
                set.remove(value);
            }

            if set.len() == 0 {
                db.remove(key);
                response.add_integer(prev_len.try_into()?);
            } else {
                let count_values_removed = prev_len - set.len();
                response.add_integer(count_values_removed.try_into()?);
            }
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            response.add_integer(0);
        }
    }

    Ok(())
}

pub(crate) fn sscan_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    response.add_error("ERR not implemented yet");
    Ok(())
}
