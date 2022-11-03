use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
    response_ext::ResponseExt,
};
use byte_string::ByteString;
use std::convert::TryInto;

pub(crate) fn rpush_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let values = &request.arguments()[1..];

    match db.get_mut(key) {
        Some(RObj::List(ref mut list)) => {
            list.extend(values.to_owned());

            response.add_integer(list.len().try_into()?);
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            db.insert(key.to_owned(), RObj::new_list_from(values.to_owned()));
            response.add_integer(values.len().try_into()?);
        }
    }

    Ok(())
}

pub(crate) fn lpush_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;
    let values = &request.arguments()[1..];

    match db.get_mut(key) {
        Some(RObj::List(ref mut list)) => {
            values.iter().for_each(|v| list.push_front(v.to_owned()));

            response.add_integer(list.len().try_into()?);
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            let len = values.len().try_into()?;
            let iter_reversed = values.iter().rev().cloned();
            db.insert(key.to_owned(), RObj::new_list_from(iter_reversed));

            response.add_integer(len);
        }
    }

    Ok(())
}

pub(crate) fn linsert_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get_mut(key) {
        Some(RObj::List(ref mut list)) => {
            let side = request.arg(1)?;
            let pivot = request.arg(2)?;

            if let Some(idx) = list.iter().position(|elem| elem == pivot) {
                let idx = match side.to_lowercase().as_ref() {
                    b"before" => idx,
                    b"after" => idx + 1,
                    _ => {
                        response.add_error("ERR syntax error");
                        return Ok(());
                    }
                };

                let value = request.arg(3)?;
                list.insert(idx, value.clone());

                response.add_integer(list.len().try_into()?);
            } else {
                response.add_integer(-1);
            }
        }
        Some(_) => response.add_reply_wrong_type(),
        None => response.add_integer(0),
    }

    Ok(())
}

pub(crate) fn rpop_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get_mut(key) {
        Some(RObj::List(ref mut list)) => {
            if let Some(value) = list.pop_back() {
                response.add_bulk_string(&value);
            } else {
                response.add_null_string();
            }
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            response.add_null_string();
        }
    }

    Ok(())
}

pub(crate) fn lpop_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get_mut(key) {
        Some(RObj::List(ref mut list)) => {
            if let Some(value) = list.pop_front() {
                response.add_bulk_string(&value);
            } else {
                response.add_null_string();
            }
        }
        Some(_) => response.add_reply_wrong_type(),
        None => {
            response.add_null_string();
        }
    }

    Ok(())
}

pub(crate) fn llen_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(RObj::List(value)) => response.add_integer(value.len().try_into()?),
        Some(_) => response.add_reply_wrong_type(),
        None => response.add_integer(0),
    }

    Ok(())
}

pub(crate) fn lindex_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(RObj::List(list)) => {
            let offset: isize = parse_arg_or_reply_with_err!(1, request, response);

            let index = match to_index(offset, list.len())?.try_into() {
                Ok(n) => n,
                Err(_) => {
                    response.add_null_string();
                    return Ok(());
                }
            };

            if let Some(value) = list.get(index) {
                response.add_bulk_string(value);
            } else {
                response.add_null_string();
            }
        }
        Some(_) => response.add_reply_wrong_type(),
        None => response.add_null_string(),
    }

    Ok(())
}

pub(crate) fn lset_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get_mut(key) {
        Some(RObj::List(list)) => {
            let offset: isize = parse_arg_or_reply_with_err!(1, request, response);

            let index = to_index(offset, list.len())?;
            if index.is_negative() {
                response.add_error("ERR index out of range");
                return Ok(());
            }

            let index: usize = index.try_into()?;

            if let Some(existing_value) = list.get_mut(index) {
                let new_value = request.arg(2)?;
                existing_value.clear();
                existing_value.extend(new_value.as_ref());
                existing_value.shrink_to_fit();

                response.add_simple_string("OK");
            } else {
                response.add_error("ERR index out of range");
            }
        }
        Some(_) => response.add_reply_wrong_type(),
        None => response.add_error("ERR no such key"),
    }

    Ok(())
}

pub(crate) fn lrange_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(RObj::List(list)) => {
            let start_offset: isize = parse_arg_or_reply_with_err!(1, request, response);
            let end_offset: isize = parse_arg_or_reply_with_err!(2, request, response);

            let start_index = to_index(start_offset, list.len())?;
            let end_index = to_index(end_offset, list.len())?;

            if start_index > end_index || end_index < 0 || start_index >= list.len().try_into()? {
                response.add_array_len(0);
                return Ok(());
            }

            let (start_index, end_index) = clamp(start_index, end_index, list.len())?;

            response.add_array_len((end_index - start_index + 1).try_into()?);

            // Take up to the requested end index (inclusive)
            let iter = list.iter().take(end_index + 1);
            // Skip until the requested start index
            let iter = iter.skip(start_index);

            for item in iter {
                response.add_bulk_string(item);
            }
        }
        Some(_) => response.add_reply_wrong_type(),
        None => response.add_array_len(0),
    }

    Ok(())
}

pub(crate) fn ltrim_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.get_mut(key) {
        Some(RObj::List(ref mut list)) => {
            let start_offset: isize = parse_arg_or_reply_with_err!(1, request, response);
            let end_offset: isize = parse_arg_or_reply_with_err!(2, request, response);

            let start_index = to_index(start_offset, list.len())?;
            let end_index = to_index(end_offset, list.len())?;

            if start_index > end_index || end_index < 0 || start_index >= list.len().try_into()? {
                db.remove(key);
            } else {
                let (start_index, mut end_index) = clamp(start_index, end_index, list.len())?;

                if start_index > 0 {
                    list.drain(0..start_index);
                    end_index -= start_index;
                }

                if end_index < list.len() - 1 {
                    list.drain((end_index + 1)..);
                }
            }

            response.add_simple_string("OK");
        }
        Some(_) => response.add_reply_wrong_type(),
        None => response.add_simple_string("OK"),
    }

    Ok(())
}

pub(crate) fn lrem_command(
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) -> Result<()> {
    let key = request.arg(0)?;

    match db.remove(key) {
        Some(RObj::List(list)) => {
            let mut to_remove: i64 = parse_arg_or_reply_with_err!(1, request, response);
            let obj = request.arg(2)?;
            let mut removed = 0;
            let mut reverse = false;

            let maybe_rev_iter: Box<dyn DoubleEndedIterator<Item = &ByteString>> = if to_remove < 0
            {
                to_remove = -to_remove;
                reverse = true;
                Box::new(list.iter().rev())
            } else {
                Box::new(list.iter())
            };

            let filtered: Vec<&ByteString> = maybe_rev_iter
                .filter(|entry| {
                    if (to_remove == 0 || removed < to_remove) && entry == &obj {
                        removed += 1;
                        false
                    } else {
                        true
                    }
                })
                .collect();

            let result_iter: Box<dyn DoubleEndedIterator<Item = &ByteString>> = if reverse {
                Box::new(filtered.iter().rev().cloned())
            } else {
                Box::new(filtered.iter().cloned())
            };

            db.insert(key.to_owned(), RObj::new_list_from(result_iter.cloned()));
            response.add_integer(removed);
        }
        Some(_) => response.add_reply_wrong_type(),
        None => response.add_integer(0),
    }

    Ok(())
}

fn to_index(offset: isize, len: usize) -> Result<isize> {
    let anchor = if offset.is_negative() {
        len.try_into()?
    } else {
        0
    };

    Ok(anchor + offset)
}

fn clamp(start: isize, end: isize, len: usize) -> Result<(usize, usize)> {
    use std::cmp::{max, min};

    let start: usize = max(0, start).try_into()?;
    let end = min(end.try_into()?, len - 1);

    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_index() -> Result<()> {
        // types
        assert_eq!(to_index(0_isize, 2_usize)?, 0_isize);

        // positive indexes are unaltered
        assert_eq!(to_index(0, 2)?, 0);
        assert_eq!(to_index(1, 2)?, 1);
        assert_eq!(to_index(2, 2)?, 2);
        assert_eq!(to_index(3, 2)?, 3);

        // negative indexes are translated relative to the end
        assert_eq!(to_index(-1, 2)?, 1);
        assert_eq!(to_index(-2, 2)?, 0);
        assert_eq!(to_index(-3, 2)?, -1);
        assert_eq!(to_index(-4, 2)?, -2);

        Ok(())
    }

    #[test]
    fn test_clamp() -> Result<()> {
        assert_eq!(clamp(0_isize, 1_isize, 2_usize)?, (0_usize, 1_usize));

        // start indexes are clamped to 0
        assert_eq!(clamp(-1, 2, 3)?, (0, 2));
        assert_eq!(clamp(0, 2, 3)?, (0, 2));
        assert_eq!(clamp(1, 2, 3)?, (1, 2));

        // end indexes are clamped to the end
        assert_eq!(clamp(0, 1, 3)?, (0, 1));
        assert_eq!(clamp(0, 2, 3)?, (0, 2));
        assert_eq!(clamp(0, 3, 3)?, (0, 2));

        Ok(())
    }
}
