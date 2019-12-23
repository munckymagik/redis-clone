use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};

use redis_clone::{
    errors::{ProtoError, Result},
    protocol,
};

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // accept connections and process them serially
    println!("Listening ...");
    for stream in listener.incoming() {
        handle_client(stream?)?;
    }
    Ok(())
}

fn handle_client(stream: TcpStream) -> Result<()> {
    let mut out_stream = stream.try_clone()?;
    let mut reader = BufReader::new(&stream);

    loop {
        // Clients send commands as a RESP Array of Bulk Strings
        let command = match protocol::decode(&mut reader) {
            Ok(value) => value,
            Err(ProtoError::ConnectionClosed) => break,
            Err(err) => {
                let msg = err.to_string();
                eprintln!("{}", msg);
                out_stream.write_all(b"-Parser error\r\n")?;
                break;
            }
        };

        println!("Command: {:?}", command);

        out_stream.write_all(b"+OK\r\n")?;
    }

    println!("Closing connection.");
    Ok(())
}
