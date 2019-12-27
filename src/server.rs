use log::{debug, info, error, warn};
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};

use crate::{
    commands,
    db::Database,
    errors::{Error, Result},
    protocol::RespError,
    request,
    response::Response,
};

pub fn serve() -> Result<()> {
    let mut db = Database::new();
    let address = "127.0.0.1:8080";
    let listener = TcpListener::bind(address)?;

    // accept connections and process them serially
    info!("Listening at {}", address);
    for stream in listener.incoming() {
        handle_client(stream?, &mut db)?;
    }
    Ok(())
}

fn handle_client(stream: TcpStream, db: &mut Database) -> Result<()> {
    let mut out_stream = stream.try_clone()?;
    let mut reader = BufReader::new(&stream);

    loop {
        let mut response = Response::new();
        let request = match request::parse(&mut reader) {
            Ok(request) => request,
            Err(Error::Resp(RespError::ConnectionClosed)) => {
                debug!("Client closed connection");
                break;
            }
            Err(Error::EmptyQuery) => {
                // Redis ignores this and continues to await a valid command
                continue;
            }
            Err(ref err) => {
                let msg = format!("ERR {}", err);
                error!("{}", msg);
                response.add_error(&msg);
                out_stream.write_all(&response.as_bytes())?;
                break;
            }
        };

        debug!("{:?}", request);

        if let Some(cmd) = commands::lookup(&request.command) {
            cmd.execute(db, &request, &mut response)?;
            out_stream.write_all(&response.as_bytes())?;
        } else {
            let msg = format!(
                "ERR unknown command `{}`, with args beginning with: {}",
                request.command,
                request.argv_to_string()
            );
            warn!("{}", msg);
            response.add_error(&msg);
            out_stream.write_all(&response.as_bytes())?;
        }
    }

    Ok(())
}
