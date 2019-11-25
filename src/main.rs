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
    let mut request_header_buf = vec![];

    if let Err(msg) = read_line(&mut reader, &mut request_header_buf) {
        eprintln!("{} while parsing line", msg);
        stream.write_all(b"-Parser error\r\n")?;
        return Ok(());
    }

    let request_header = match parse_header(&request_header_buf) {
        Ok(header) => header,
        Err(msg) => {
            eprintln!("{} from header str {:?}", msg, request_header_buf);
            stream.write_all(b"-Parser error\r\n")?;
            return Ok(());
        }
    };

    println!("    {}", request_header);

    if request_header.type_sym != b'*' {
        eprintln!("request is not an array");
        stream.write_all(b"-Parser error\r\n")?;
        return Ok(());
    }

    let mut req_array: Vec<(Header, String)> = Vec::new();

    for _ in 0..request_header.len {
        let mut elem_header_buf: Vec<u8> = Vec::new();

        if let Err(msg) = read_line(&mut reader, &mut elem_header_buf) {
            eprintln!("{} while reading elem header line", msg);
            stream.write_all(b"-Parser error\r\n")?;
            return Ok(());
        }

        let elem_resp_header = match parse_header(&elem_header_buf) {
            Ok(header) => header,
            Err(msg) => {
                eprintln!("{} from elem header string {:?}", msg, elem_header_buf);
                stream.write_all(b"-Parser error\r\n")?;
                return Ok(());
            }
        };

        println!("    {}", elem_resp_header);

        let mut elem_value_buf = vec![0; elem_resp_header.len + 2];
        reader.read_exact(&mut elem_value_buf)?;
        let elem_value_str = std::str::from_utf8(&elem_value_buf[..elem_resp_header.len])?;
        println!("    value({:?})", elem_value_str);
        req_array.push((elem_resp_header, elem_value_str.to_owned()));
    }

    println!("    \"{:?}\"", request_header);
    for pair in req_array {
        println!("      \"{:?}\"", pair);
    }

    stream.write_all(b"+OK\r\n")?;

    println!("Closing connection.");
    Ok(())
}

fn read_line(reader: &mut impl BufRead, buffer: &mut Vec<u8>) -> Result<()> {
    let num_bytes = reader.read_until(b'\n', buffer)?;
    if num_bytes < 2 {
        return Err("line too short".into());
    }

    let tail = buffer.drain((num_bytes - 2)..num_bytes);
    debug_assert_eq!(tail.collect::<Vec<u8>>(), b"\r\n");

    Ok(())
}

fn parse_header(line: &[u8]) -> Result<Header> {
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
