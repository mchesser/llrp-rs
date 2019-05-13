use std::{convert::TryInto, fmt, io};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InvalidType(u16),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "{}", e),
            Error::InvalidType(type_id) => write!(f, "Invalid type id: {}", type_id),
        }
    }
}
impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::IoError(e) => e,
            Error::InvalidType(type_id) => {
                io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid type id: {}", type_id))
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait LLRPDecodable: Sized {
    const ID: u16 = 0;

    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        Err(io::Error::new(io::ErrorKind::Other, "Unimplemented").into())
    }

    fn id(&self) -> u16 {
        Self::ID
    }
}

impl LLRPDecodable for () {}
impl LLRPDecodable for bool {}
impl LLRPDecodable for i8 {}
impl LLRPDecodable for i16 {}
impl LLRPDecodable for i32 {}
impl LLRPDecodable for i64 {}

impl LLRPDecodable for u8 {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() < 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }
        Ok((data[0], &data[1..]))
    }
}

impl LLRPDecodable for u16 {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() < 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }
        let value = u16::from_be_bytes([data[0], data[1]]);
        Ok((value, &data[2..]))
    }
}

impl LLRPDecodable for u32 {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() < 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }
        let value = u32::from_be_bytes(data[..4].try_into().unwrap());
        Ok((value, &data[4..]))
    }
}

impl LLRPDecodable for u64 {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() < 8 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }
        let value = u64::from_be_bytes(data[..8].try_into().unwrap());
        Ok((value, &data[8..]))
    }
}

impl LLRPDecodable for String {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() < 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }
        let length = u16::from_be_bytes([data[0], data[1]]) as usize;
        if data.len() < 2 + length {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }

        let string = String::from_utf8(data[2..][..length].into())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        eprintln!("{}", string);
        Ok((string, &data[2 + length..]))
    }
}

impl<T: LLRPDecodable> LLRPDecodable for Option<T> {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() == 0 {
            return Ok((None, data));
        }

        match <T as LLRPDecodable>::decode(data) {
            Ok((field, rest)) => Ok((Some(field), rest)),
            Err(Error::InvalidType(_)) => Ok((None, data)),
            Err(e) => return Err(e),
        }
    }
}

impl<T: LLRPDecodable> LLRPDecodable for Vec<T> {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        let mut output = vec![];

        let mut rest = data;
        loop {
            match <T as LLRPDecodable>::decode(rest) {
                Ok((field, new_rest)) => {
                    output.push(field);
                    rest = new_rest;
                }
                Err(Error::InvalidType(_)) => break,
                Err(e) => return Err(e),
            }
        }

        Ok((output, rest))
    }
}

impl<T: LLRPDecodable> LLRPDecodable for Box<T> {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        let (result, rest) = <T as LLRPDecodable>::decode(data)?;
        Ok((Box::new(result), rest))
    }
}
