use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn call(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let key = request.arg(0).unwrap();

    if db.contains_key(key) {
        let value = &db[key];

        let value: i64 = match value.parse() {
            Ok(v) => v,
            Err(_) => {
                response.add_error("ERR value is not an integer or out of range");
                return Ok(());
            }
        };

        if let Some(value) = value.checked_add(1) {
            db.insert(key.to_string(), value.to_string());
            response.add_integer(value);
        } else {
            response.add_error("ERR increment or decrement would overflow")
        }
    } else {
        db.insert(key.to_string(), 1.to_string());
        response.add_integer(1);
    }

    Ok(())
}
