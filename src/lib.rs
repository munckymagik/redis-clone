#![forbid(unsafe_code)]

pub mod server;

#[macro_use]
mod macros;

mod commands;
mod db;
mod errors;
mod protocol;
mod request;
mod response;
mod response_ext;
