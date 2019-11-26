/// Implement the RESP protool https://redis.io/topics/protocol

use std::fmt;
use std::io::{BufRead, Read};

use super::errors::{Result, ServerError};

#[derive(Debug)]
pub struct Header {
    type_sym: u8,
    len: usize,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "header({}, {:?})", char::from(self.type_sym), self.len)
    }
}

pub fn read_array_of_bulkstrings(reader: &mut impl BufRead) -> Result<Vec<(Header, String)>> {
    let mut request_header_buf = vec![];

    if let Err(msg) = read_line(reader, &mut request_header_buf) {
        let err_msg = format!("{} while parsing line", msg);
        return Err(err_msg.into());
    }

    let request_header = match parse_header(&request_header_buf) {
        Ok(header) => header,
        Err(msg) => {
            let err_msg = format!("{} from header str {:?}", msg, request_header_buf);
            return Err(err_msg.into());
        }
    };

    println!("    {}", request_header);

    if request_header.type_sym != b'*' {
        let err_msg = format!("request is not an array");
        return Err(err_msg.into());
    }

    let mut req_array = Vec::new();

    for _ in 0..request_header.len {
        let mut elem_header_buf: Vec<u8> = Vec::new();

        if let Err(msg) = read_line(reader, &mut elem_header_buf) {
            let err_msg = format!("{} while reading elem header line", msg);
            return Err(err_msg.into());
        }

        let elem_resp_header = match parse_header(&elem_header_buf) {
            Ok(header) => header,
            Err(msg) => {
                let err_msg = format!(
                    "{} while reading elem header string {:?}",
                    msg, elem_header_buf
                );
                return Err(err_msg.into());
            }
        };
        println!("    {}", elem_resp_header);

        let elem_value_str = match read_bulk_string(reader, &elem_resp_header) {
            Ok(value) => value,
            Err(msg) => {
                let err_msg = format!("{} while reading elem value string", msg);
                return Err(err_msg.into());
            }
        };
        println!("    value({:?})", elem_value_str);

        req_array.push((elem_resp_header, elem_value_str));
    }

    Ok(req_array)
}

fn read_line(reader: &mut impl BufRead, buffer: &mut Vec<u8>) -> Result<()> {
    let num_bytes = reader.read_until(b'\n', buffer)?;

    if num_bytes == 0 {
        return Err(ServerError::EmptyRead);
    } else if num_bytes < 2 {
        return Err("line too short".into());
    }

    let tail = buffer.drain((num_bytes - 2)..num_bytes);
    debug_assert_eq!(tail.collect::<Vec<u8>>(), b"\r\n");

    Ok(())
}

fn read_bulk_string(reader: &mut impl Read, header: &Header) -> Result<String> {
    let mut buffer = vec![0; header.len + 2];
    reader.read_exact(&mut buffer)?;
    let value_str = std::str::from_utf8(&buffer[..header.len])?;

    Ok(value_str.to_owned())
}

fn parse_header(line: &[u8]) -> Result<Header> {
    let (&type_sym, len_str) = line
        .split_first()
        .ok_or_else(|| ServerError::from("Error parsing resp header structure"))?;
    let len_str = std::str::from_utf8(len_str)?;
    let len: usize = len_str.parse().map_err(|e| {
        let msg = format!("{} while parsing {:?}", e, len_str);
        ServerError::from(msg)
    })?;

    Ok(Header { type_sym, len })
}
