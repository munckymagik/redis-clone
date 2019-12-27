use crate::{db::Database, errors::Result, request::Request, response::Response};
use std::sync::{Arc, Mutex};

pub(crate) fn call(db: Arc<Mutex<Database>>, request: &Request, response: &mut Response) -> Result<()> {
    if let Some(key) = request.arg(0) {
        if let Some(value) = request.arg(1) {
            let mut db = db.lock().unwrap();
            db.insert(key.to_owned(), value.to_owned());
            response.add_simple_string("OK");
        } else {
            response.add_error("ERR missing value");
        }
    } else {
        response.add_error("ERR missing key");
    }

    Ok(())
}
