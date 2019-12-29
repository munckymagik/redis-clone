//! Implementation of the RESP protool https://redis.io/topics/protocol

mod decode2;
mod errors;

pub use decode2::decode as decode2;
pub use errors::{RespError, RespResult};

const DEPTH_LIMIT: usize = 420; // TODO reduced from 512 to avoid stack overflow in test
const MAX_ARRAY_SIZE: usize = 1024 * 1024;
const MAX_BULK_STR_SIZE: usize = 512 * 1024 * 1024;
const MAX_LINE_LENGTH: usize = 64 * 1024;
const LF: u8 = b'\n';
const CRLF: &[u8] = b"\r\n";
