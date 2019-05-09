pub mod messages;
pub mod deserializer;
pub mod types;

use std::io;

pub enum Error {
    Io(io::Error),
    BadResponse(String),
}

pub type Result<T> = std::result::Result<T, Error>;
