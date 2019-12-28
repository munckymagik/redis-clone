use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn call(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    if let Some(key) = request.arg(0) {
        match db.remove(key) {
            Some(_) => response.add_integer(1),
            None => response.add_integer(0),
        }
    } else {
        response.add_error("ERR missing key");
    }

    Ok(())
}
