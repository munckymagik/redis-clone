use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn call(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    if let Some(key) = request.arg(0) {
        match db.get(key) {
            Some(value) => {
                response.add_bulk_string(value);
            }
            None => response.add_null_string(),
        }
    } else {
        response.add_error("ERR missing key");
    }

    Ok(())
}
