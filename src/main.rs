use std::collections::HashMap;
use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};

use redis_clone::protocol::{self, RespError, RespResult, RespVal};

fn main() -> RespResult<()> {
    let mut db: HashMap<String, String> = HashMap::new();
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // accept connections and process them serially
    println!("Listening ...");
    for stream in listener.incoming() {
        handle_client(stream?, &mut db)?;
    }
    Ok(())
}

fn handle_client(stream: TcpStream, db: &mut HashMap<String, String>) -> RespResult<()> {
    let mut out_stream = stream.try_clone()?;
    let mut reader = BufReader::new(&stream);

    loop {
        // Clients send commands as a RESP Array of Bulk Strings
        let request = match protocol::decode(&mut reader) {
            Ok(value) => value,
            Err(RespError::ConnectionClosed) => break,
            Err(err) => {
                let msg = err.to_string();
                eprintln!("{}", msg);
                out_stream.write_all(b"-Parser error\r\n")?;
                break;
            }
        };

        println!("Request: {:?}", request);

        if let RespVal::Array(Some(argv)) = request {
            if let Some(RespVal::BulkString(Some(command))) = argv.get(0) {
                println!("Command: {}", command);
                match command.as_ref() {
                    "set" => {
                        if let Some(RespVal::BulkString(Some(key))) = argv.get(1) {
                            if let Some(RespVal::BulkString(Some(value))) = argv.get(2) {
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
                        if let Some(RespVal::BulkString(Some(key))) = argv.get(1) {
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
                        if let Some(RespVal::BulkString(Some(key))) = argv.get(1) {
                            match db.remove(key) {
                                Some(_) => out_stream.write_all(b":1\r\n")?,
                                None => out_stream.write_all(b":0\r\n")?,
                            }
                        } else {
                            out_stream.write_all(b"-ERR missing key\r\n")?;
                        }
                    }
                    _ => todo!(),
                };
            } else {
                out_stream.write_all(b"-ERR missing command\r\n")?;
                println!("ERR missing command");
            }
        } else {
            println!("ERR request is not an array of bulk string");
        }
    }

    println!("Closing connection.");
    Ok(())
}
