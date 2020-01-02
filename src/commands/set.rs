use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn call(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let key = request.arg(0).unwrap();
    let value = request.arg(1).unwrap();
    db.insert(key.to_owned(), value.to_owned());
    response.add_simple_string("OK");

    Ok(())
}
