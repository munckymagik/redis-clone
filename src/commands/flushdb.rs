use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn call(db: &mut Database, _request: &Request, response: &mut Response) -> Result<()> {
    // Clears all the key-values but retains memory
    db.clear();

    // Releases memory
    db.shrink_to_fit();

    response.add_simple_string("OK");

    Ok(())
}
