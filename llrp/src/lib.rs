pub mod deserializer;
pub mod messages;
pub mod parameters;

#[cfg(test)]
mod tests;

use std::io;

pub enum Error {
    Io(io::Error),
    BadResponse(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub use llrp_common::{LLRPMessage, LLRPDecodable, TvDecodable, TlvDecodable, parse_tlv_header};
