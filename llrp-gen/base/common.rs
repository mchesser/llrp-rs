use std::{convert::TryInto, fmt, io};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InvalidData,
    InvalidType(u16),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "{}", e),
            Error::InvalidData => write!(f, "Invalid data"),
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
            Error::InvalidData => {
                io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid data"))
            }
            Error::InvalidType(type_id) => {
                io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid type id: {}", type_id))
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Divides one slice into two at an index, returning an error if the index is greater than the
/// length of the slice.
pub fn split_at_checked(data: &[u8], mid: usize) -> Result<(&[u8], &[u8])> {
    if data.len() < mid {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
    }
    Ok(data.split_at(mid))
}

/// Divides one slice into two, where the length of the first slice is given by a be_u16 encoded
/// length, returning an error array is not long enough
pub fn split_with_u16_length(data: &[u8]) -> Result<(&[u8], &[u8])> {
    let (len, data) = split_at_checked(data, 2)
        .map(|(data, rest)| (u16::from_be_bytes(data.try_into().unwrap()), rest))?;
    split_at_checked(data, len as usize)
}

/// Ensures that all bytes were consumed when parsing the struct fields
/// TODO: consider adding a feature to run in `relaxed` mode where this error is ignored
pub fn validate_consumed(data: &[u8]) -> Result<()> {
    if data.is_empty() {
        return Err(
            io::Error::new(io::ErrorKind::InvalidData, "Parameter had trailing bytes").into()
        );
    }
    Ok(())
}

pub trait LLRPMessage: Sized {
    const ID: u16;

    fn decode(data: &[u8]) -> Result<(Self, &[u8])>;

    fn id(&self) -> u16 {
        Self::ID
    }
}

pub trait LLRPDecodable: Sized {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])>;
}

impl LLRPDecodable for bool {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        let (data, rest) = split_at_checked(data, 1)?;
        Ok((data[0] != 0, rest))
    }
}

macro_rules! impl_llrp_decodable_primitive {
    ($ty: ty) => {
        impl LLRPDecodable for $ty {
            fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
                split_at_checked(data, std::mem::size_of::<$ty>())
                    .map(|(data, rest)| (Self::from_be_bytes(data.try_into().unwrap()), rest))
            }
        }
    };
}
impl_llrp_decodable_primitive!(i8);
impl_llrp_decodable_primitive!(u8);
impl_llrp_decodable_primitive!(u16);
impl_llrp_decodable_primitive!(i16);
impl_llrp_decodable_primitive!(u32);
impl_llrp_decodable_primitive!(u64);

impl LLRPDecodable for [u8; 12] {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        split_at_checked(data, 12).map(|(data, rest)| (data.try_into().unwrap(), rest))
    }
}

impl LLRPDecodable for String {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        let (data, rest) = split_with_u16_length(data)?;

        let string = String::from_utf8(data.into())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        eprintln!("{}", string);
        Ok((string, rest))
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct BitArray {
    pub bytes: Vec<u8>,
}

impl BitArray {
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> BitArray {
        BitArray { bytes: bytes.into() }
    }
}

impl LLRPDecodable for BitArray {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        let (num_bits, data) = split_at_checked(data, 2)
            .map(|(data, rest)| (u16::from_be_bytes(data.try_into().unwrap()) as usize, rest))?;

        let (data, rest) = split_at_checked(data, num_bits / 8)?;

        Ok((BitArray { bytes: data.into() }, rest))
    }
}

pub trait FromBits {
    fn from_bits(bits: u32) -> Self;
}

impl FromBits for bool {
    fn from_bits(bits: u32) -> Self {
        (bits & 1) != 0
    }
}

impl FromBits for u8 {
    fn from_bits(bits: u32) -> Self {
        bits as u8
    }
}

impl FromBits for u16 {
    fn from_bits(bits: u32) -> Self {
        bits as u16
    }
}

pub fn get_tlv_message_type(data: &[u8]) -> Result<u16> {
    if data.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
    }
    Ok(u16::from_be_bytes([data[0], data[1]]) & 0b11_1111_1111)
}

pub fn parse_tlv_header(data: &[u8], target_type: u16) -> Result<(&[u8], &[u8])> {
    // The first two bytes consist of [6-bit resv, 10-bit message type]
    let (type_bytes, data) = split_at_checked(data, 2)?;
    let type_ = u16::from_be_bytes([type_bytes[0], type_bytes[1]]) & 0b11_1111_1111;

    eprintln!("type = {}", type_);
    if type_ != target_type {
        return Err(Error::InvalidType(type_));
    }

    split_with_u16_length(data)
}

pub trait TlvDecodable: Sized {
    const ID: u16 = 0;
    fn decode_tlv(_data: &[u8]) -> Result<(Self, &[u8])> {
        unimplemented!()
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

pub trait LLRPEnumeration: Sized {
    fn from_value<T: Into<u32>>(value: T) -> Result<Self>;

    fn from_vec<T: Into<u32>>(value: Vec<T>) -> Result<Vec<Self>> {
        value.into_iter().map(|x| Self::from_value(x.into())).collect()
    }
}

impl<E: LLRPEnumeration> crate::FromBits for E {
    fn from_bits(bits: u32) -> Self {
        Self::from_value(bits).unwrap()
    }
}

#[derive(Default)]
pub struct BitContainer {
    pub bits: u32,
    pub valid_bits: u8,
}

impl BitContainer {
    pub fn read_bits<'a>(&mut self, mut data: &'a [u8], num_bits: u8) -> Result<(u32, &'a [u8])> {
        while self.valid_bits < num_bits {
            if data.is_empty() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
            }
            self.bits = (self.bits << 8) | data[0] as u32;
            self.valid_bits += 8;
            data = &data[1..];
        }

        let out = self.bits & ((1 << num_bits) - 1);
        self.bits = self.bits >> num_bits;
        self.valid_bits -= num_bits;

        Ok((out, data))
    }
}
