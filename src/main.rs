use std::collections::HashMap;
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};

use redis_clone::protocol::{self, RespError};
use redis_clone::{lookup_command, process_query, Error};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut db: HashMap<String, String> = HashMap::new();
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // accept connections and process them serially
    println!("Listening ...");
    for stream in listener.incoming() {
        handle_client(stream?, &mut db)?;
    }
    Ok(())
}

fn handle_client(stream: TcpStream, db: &mut HashMap<String, String>) -> Result<()> {
    let mut out_stream = stream.try_clone()?;
    let mut reader = BufReader::new(&stream);

    loop {
        // Clients send commands as a RESP Array of Bulk Strings
        let query = match protocol::decode(&mut reader) {
            Ok(value) => value,
            Err(RespError::ConnectionClosed) => {
                println!("Client closed connection");
                break;
            }
            Err(ref err) => {
                println!("Reading from client: {}", err);
                write!(out_stream, "-ERR Parser error: {}\r\n", err)?;
                break;
            }
        };

        let multi_cmd = match process_query(query) {
            Ok(multi_cmd) => multi_cmd,
            Err(Error::EmptyQuery) => {
                // Redis ignores this and continues to await a valid command
                continue;
            }
            Err(ref err) => {
                println!("{}", err);
                write!(out_stream, "-ERR {}\r\n", err)?;
                break;
            }
        };

        println!("{:?}", multi_cmd);

        match multi_cmd.command.as_ref() {
            "set" => {
                if let Some(key) = multi_cmd.argv.get(0) {
                    if let Some(value) = multi_cmd.argv.get(1) {
                        db.insert(key.to_owned(), value.to_owned());
                        out_stream.write_all(b"+OK\r\n")?;
                    } else {
                        out_stream.write_all(b"-ERR missing value\r\n")?;
                    }
                } else {
                    out_stream.write_all(b"-ERR missing key\r\n")?;
                }
            }
            "get" => {
                if let Some(key) = multi_cmd.argv.get(0) {
                    match db.get(key) {
                        Some(value) => {
                            let out = format!("${}\r\n{}\r\n", value.len(), value);
                            out_stream.write_all(out.as_bytes())?;
                        }
                        None => out_stream.write_all(b"$-1\r\n")?,
                    }
                } else {
                    out_stream.write_all(b"-ERR missing key\r\n")?;
                }
            }
            "del" => {
                if let Some(key) = multi_cmd.argv.get(0) {
                    match db.remove(key) {
                        Some(_) => out_stream.write_all(b":1\r\n")?,
                        None => out_stream.write_all(b":0\r\n")?,
                    }
                } else {
                    out_stream.write_all(b"-ERR missing key\r\n")?;
                }
            }
            _ => {
                if let Some(cmd) = lookup_command(&multi_cmd.command) {
                    let reply = (cmd.proc)(&multi_cmd.argv)?;
                    out_stream.write_all(&reply.as_bytes())?;
                } else {
                    let args = multi_cmd
                        .argv
                        .iter()
                        .map(|v| format!("`{}`,", v))
                        .collect::<Vec<String>>()
                        .join(" ");
                    write!(
                        out_stream,
                        "-ERR unknown command `{}`, with args beginning with: {}\r\n",
                        multi_cmd.command, args
                    )?;
                }
            }
        };
    }

    Ok(())
}
