use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn call(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let mut count = 0;

    for key in request.arguments() {
        if db.contains_key(key) {
            count += 1;
        }
    }

    response.add_integer(count);

    Ok(())
}