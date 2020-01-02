use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn call(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let key = request.arg(0).unwrap();

    match db.get(key) {
        Some(value) => {
            response.add_bulk_string(value);
        }
        None => response.add_null_string(),
    }

    Ok(())
}
