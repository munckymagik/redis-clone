use crate::{
    db::{Database, RObj},
    errors::Result,
    request::Request,
    response::Response,
};

pub(crate) fn call(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let key = request.arg(0)?;

    match db.get(key) {
        Some(RObj::String(value)) => {
            response.add_bulk_string(value);
        }
        Some(_) => response.add_null_string(),
        None => response.add_null_string(),
    }

    Ok(())
}
