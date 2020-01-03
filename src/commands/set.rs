use crate::{db::Database, errors::Result, request::Request, response::Response};

pub(crate) fn call(db: &mut Database, request: &Request, response: &mut Response) -> Result<()> {
    let key = request.arg(0).unwrap();
    let value = request.arg(1).unwrap();
    let mut nx = false;
    let mut xx = false;

    for arg in &request.arguments()[2..] {
        match arg.to_lowercase().as_ref() {
            "nx" if !xx => nx = true,
            "xx" if !nx => xx = true,
            _ => {
                response.add_error("ERR syntax error");
                return Ok(());
            }
        }
    }

    if nx && db.contains_key(key) || xx && !db.contains_key(key) {
        response.add_null_string();
        return Ok(());
    }

    db.insert(key.to_owned(), value.to_owned());
    response.add_simple_string("OK");

    Ok(())
}
