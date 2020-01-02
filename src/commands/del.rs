use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn call(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let key = request.arg(0).unwrap();

    match db.remove(key) {
        Some(_) => response.add_integer(1),
        None => response.add_integer(0),
    }

    Ok(())
}
