use crate::{db::Database, errors::Result, request::Request, response::Response};
use globber::Pattern;
use std::convert::TryInto;

pub(crate) fn call(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let pattern = request.arg(0).unwrap();
    let matcher = match Pattern::new(pattern) {
        Ok(m) => m,
        Err(_) => {
            response.add_array_len(0);
            return Ok(());
        }
    };

    let results: Vec<_> = if pattern == "*" {
        db.keys().collect()
    } else {
        db.keys().filter(|key| matcher.matches(key)).collect()
    };

    response.add_array_len(results.len().try_into().unwrap());
    for key in results {
        response.add_bulk_string(key);
    }

    Ok(())
}
