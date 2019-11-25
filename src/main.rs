use std::fmt;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

type BoxedError = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, BoxedError>;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // accept connections and process them serially
    println!("Listening ...");
    for stream in listener.incoming() {
        handle_client(stream?)?;
    }
    Ok(())
}

#[derive(Debug)]
struct Header {
    type_sym: u8,
    len: usize,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "header({}, {:?})", char::from(self.type_sym), self.len)
    }
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut buffer: Vec<u8> = Vec::new();

    let num_bytes = reader.read_until(b'\n', &mut buffer)?;
    println!("    buffer({:?})", buffer);
    // TODO what if less than 2 bytes?
    let line: &[u8] = &buffer[..num_bytes - 2];

    println!("    line({:?})", line);
    let resp_header = match parse_resp_header(line) {
        Ok(header) => header,
        Err(msg) => {
            eprintln!("{} from header str {:?}", msg, line);
            stream.write_all(b"-Parser error\r\n")?;
            return Ok(());
        }
    };

    println!("    {}", resp_header);

    if resp_header.type_sym == b'*' {
        let mut req_array: Vec<(Header, String)> = Vec::new();

        for _ in 0..resp_header.len {
            let mut elem_header_buf: Vec<u8> = Vec::new();
            let num_bytes = reader.read_until(b'\n', &mut elem_header_buf)?;
            // TODO what if less than 2 bytes?
            let elem_header_line: &[u8] = &elem_header_buf[..num_bytes - 2];

            let elem_resp_header = match parse_resp_header(elem_header_line) {
                Ok(header) => header,
                Err(msg) => {
                    eprintln!("{} from sub-header string {:?}", msg, elem_header_line);
                    stream.write_all(b"-Parser error\r\n")?;
                    return Ok(());
                }
            };

            println!("    {}", resp_header);

            let mut elem_value_buf = vec![0; elem_resp_header.len + 2];
            reader.read_exact(&mut elem_value_buf)?;
            let elem_value_str = std::str::from_utf8(&elem_value_buf[..elem_resp_header.len])?;
            println!("    value({:?})", elem_value_str);
            req_array.push((elem_resp_header, elem_value_str.to_owned()));
        }

        println!("    \"{:?}\"", resp_header);
        for pair in req_array {
            println!("      \"{:?}\"", pair);
        }
    } else {
        stream.write_all(b"-Sorry I did not understand\r\n")?;
        return Ok(());
    }

    stream.write_all(b"+OK\r\n")?;

    println!("Closing connection.");
    Ok(())
}

fn parse_resp_header(line: &[u8]) -> Result<Header> {
    let (&type_sym, len_str) = line
        .split_first()
        .ok_or_else(|| BoxedError::from("Error parsing resp header structure"))?;
    let len_str = std::str::from_utf8(len_str)?;
    let len: usize = len_str.parse().map_err(|e| {
        let msg = format!("{} while parsing {:?}", e, len_str);
        BoxedError::from(msg)
    })?;

    Ok(Header { type_sym, len })
}
