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

pub trait LLRPMessage: Sized {
    const ID: u16;

    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        Err(io::Error::new(io::ErrorKind::Other, "Unimplemented").into())
    }

    fn id(&self) -> u16 {
        Self::ID
    }
}

pub trait LLRPDecodable: Sized {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        Err(io::Error::new(io::ErrorKind::Other, "Unimplemented").into())
    }
}

impl LLRPDecodable for bool {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() < 1 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }
        Ok((data[0] != 0, &data[1..]))
    }
}

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

impl LLRPDecodable for [u8; 12] {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() < 12 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }
        Ok((data[..12].try_into().unwrap(), &data[12..]))
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

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct BitArray {
    pub bytes: Vec<u8>,
}

impl LLRPDecodable for BitArray {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() < 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }

        let num_bits = u16::from_be_bytes([data[0], data[1]]) as usize;
        let num_bytes = num_bits / 8;

        if data.len() < 2 + num_bytes {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }

        let array = BitArray {
            bytes: data[2..][..num_bytes].into(),
        };
        Ok((array, &data[2 + num_bytes..]))
    }
}

pub fn parse_tlv_header(data: &[u8], target_type: u16) -> Result<(&[u8], usize)> {
    if data.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
    }
    eprintln!("data = {:02x?}", data);

    // [6-bit resv, 10-bit message type]
    let type_ = u16::from_be_bytes([data[0], data[1]]) & 0b11_1111_1111;
    eprintln!("type = {}", type_);
    if type_ != target_type {
        return Err(Error::InvalidType(type_));
    }

    // 16-bit length
    let len = u16::from_be_bytes([data[2], data[3]]) as usize;
    if len > data.len() {
        // Length was larger than the remaining data
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
    }

    Ok((&data[4..len], len))
}

pub trait TlvDecodable: Sized {
    const ID: u16 = 0;
    fn decode_tlv(_data: &[u8]) -> Result<(Self, &[u8])> {
        unimplemented!()
    }
}

impl<T: TlvDecodable> LLRPDecodable for T {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        <T as TlvDecodable>::decode_tlv(data)
    }
}

impl<T: TlvDecodable> TlvDecodable for Option<T> {
    fn decode_tlv(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() == 0 {
            return Ok((None, data));
        }

        match <T as TlvDecodable>::decode_tlv(data) {
            Ok((field, rest)) => Ok((Some(field), rest)),
            Err(Error::InvalidType(_)) => Ok((None, data)),
            Err(e) => return Err(e),
        }
    }
}

impl<T: TlvDecodable> TlvDecodable for Box<T> {
    fn decode_tlv(data: &[u8]) -> Result<(Self, &[u8])> {
        let (result, rest) = <T as TlvDecodable>::decode_tlv(data)?;
        Ok((Box::new(result), rest))
    }
}

impl<T: TlvDecodable> TlvDecodable for Vec<T> {
    fn decode_tlv(data: &[u8]) -> Result<(Self, &[u8])> {
        let mut output = vec![];

        let mut rest = data;
        while rest.len() > 0 {
            match <T as TlvDecodable>::decode_tlv(rest) {
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

pub trait TvDecodable: Sized {
    fn decode_tv(data: &[u8], id: u8) -> Result<(Self, &[u8])>;
}

impl<T: LLRPDecodable> TvDecodable for Option<T> {
    fn decode_tv(data: &[u8], id: u8) -> Result<(Self, &[u8])> {
        if data.len() < 2 {
            return Ok((None, data));
        }

        let found_type = data[0] & 0x7F;
        if ((data[0] & 0x80) == 0) || found_type != id {
            return Ok((None, data));
        }

        let (data, rest) = <T as LLRPDecodable>::decode(&data[1..])?;
        Ok((Some(data), rest))
    }
}
