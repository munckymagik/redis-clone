use std::io::BufReader;
use std::io::{BufRead, Write};
use std::net::{TcpListener, TcpStream};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // accept connections and process them serially
    println!("Listening ...");
    for stream in listener.incoming() {
        handle_client(stream?)?;
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut req_array_len: usize = 0;
    let mut buffer: Vec<u8> = Vec::new();

    let num_bytes = reader.read_until(b'\n', &mut buffer)?;
    println!("    buffer({:?})", buffer);
    let line: &[u8] = &buffer[..num_bytes-2];

    println!("    line({:?})", line);
    if line.starts_with(b"*") {
        let len_str = std::str::from_utf8(&line[1..])?;
        req_array_len = match len_str.parse() {
            Ok(n) => n,
            _ => {
                stream.write_all(b"-Parser error\r\n")?;
                return Ok(());
            }
        };

        println!("    array({})", req_array_len);
        for line in reader.lines().take(req_array_len * 2) {
            println!("      \"{}\"", line?);
        }
    } else {
        stream.write_all(b"-Sorry I did not understand\r\n")?;
        return Ok(());
    }

    stream.write_all(b"+OK\r\n")?;

    println!("Done.");
    Ok(())
}
