use std::{convert::TryInto, fmt, io};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InsufficientData { needed: usize, remaining: usize },
    TrailingBits(usize),
    TrailingBytes(usize),
    TlvParameterLengthInvalid(u16),
    InvalidType(u16),
    InvalidTvType(u8),
    InvalidVariant(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "{}", e),
            Error::InsufficientData { needed, remaining } => write!(
                f,
                "Insufficient data: {} bytes needed, but only {} remaining",
                needed, remaining
            ),
            Error::TrailingBytes(len) => write!(f, "{} trailing bytes", len),
            Error::TrailingBits(len) => write!(f, "{} trailing bits", len),
            Error::TlvParameterLengthInvalid(len) => {
                write!(f, "Invalid length for TLV parameter: {}", len)
            }
            Error::InvalidType(type_id) => write!(f, "Invalid type num: {}", type_id),
            Error::InvalidTvType(type_id) => write!(f, "Invalid type num: {}", type_id),
            Error::InvalidVariant(value) => write!(f, "Invalid variant: {}", value),
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
        if let Error::IoError(e) = err {
            e
        }
        else {
            io::Error::new(io::ErrorKind::InvalidInput, format!("{}", err))
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Divides one slice into two at an index, returning an error if the index is greater than the
/// length of the slice.
pub(crate) fn split_at_checked(data: &[u8], mid: usize) -> Result<(&[u8], &[u8])> {
    if data.len() < mid {
        return Err(Error::InsufficientData { needed: mid, remaining: data.len() });
    }
    Ok(data.split_at(mid))
}

/// Divides one slice into two, where the length of the first slice is given by a be_u16 encoded
/// length, returning an error array is not long enough
pub(crate) fn split_with_u16_length(data: &[u8]) -> Result<(&[u8], &[u8])> {
    let (len, data) = split_at_checked(data, 2)
        .map(|(data, rest)| (u16::from_be_bytes(data.try_into().unwrap()), rest))?;
    split_at_checked(data, len as usize)
}

/// Ensures that all bytes were consumed when parsing the struct fields
/// TODO: consider adding a feature to run in `relaxed` mode where this error is ignored
pub(crate) fn validate_consumed(data: &[u8]) -> Result<()> {
    if !data.is_empty() {
        return Err(Error::TrailingBytes(data.len()));
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

impl<T: LLRPDecodable> LLRPDecodable for Option<T> {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        use Error::*;

        match <T as LLRPDecodable>::decode(data) {
            Ok((value, rest)) => Ok((Some(value), rest)),
            Err(InvalidType(..)) | Err(InvalidTvType(..)) | Err(InsufficientData { .. }) => {
                Ok((None, data))
            }
            Err(other) => Err(other),
        }
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

pub(crate) trait FromBits {
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
        return Err(Error::InsufficientData { needed: 2, remaining: data.len() });
    }
    // The first two bytes consist of [6-bit resv, 10-bit message type]
    Ok(u16::from_be_bytes([data[0], data[1]]) & 0b11_1111_1111)
}

pub fn parse_tlv_header(data: &[u8], target_type: u16) -> Result<(&[u8], &[u8])> {
    let (type_bytes, data) = split_at_checked(data, 2)?;
    let type_ = get_tlv_message_type(type_bytes)?;

    if type_ != target_type {
        return Err(Error::InvalidType(type_));
    }

    // Decode the parameter length field.
    // Note: The length field covers the entire parameter, including the 4 header bytes at the start
    let (param_length, data) = u16::decode(data)?;
    let len = param_length as usize;
    if len < 4 || len - 4 > data.len() {
        return Err(Error::TlvParameterLengthInvalid(param_length));
    }

    split_at_checked(data, len - 4)
}

pub trait TlvDecodable: Sized {
    const ID: u16 = 0;

    fn decode_tlv(_data: &[u8]) -> Result<(Self, &[u8])>;

    fn check_type(ty: u16) -> bool {
        ty == Self::ID
    }
}

impl<T: TlvDecodable> TlvDecodable for Option<T> {
    fn decode_tlv(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() == 0 {
            return Ok((None, data));
        }

        match get_tlv_message_type(data) {
            Ok(ty) if T::check_type(ty) => {
                let (field, rest) = <T as TlvDecodable>::decode_tlv(data)?;
                Ok((Some(field), rest))
            }
            _ => Ok((None, data)),
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
            match get_tlv_message_type(rest) {
                Ok(ty) if T::check_type(ty) => {
                    let (field, new_rest) = <T as TlvDecodable>::decode_tlv(rest)?;
                    output.push(field);
                    rest = new_rest;
                }
                _ => break,
            }
        }

        Ok((output, rest))
    }
}

pub trait TvDecodable: Sized {
    fn decode_tv(data: &[u8], id: u8) -> Result<(Self, &[u8])>;
}

impl<T: LLRPDecodable> TvDecodable for T {
    fn decode_tv(data: &[u8], id: u8) -> Result<(Self, &[u8])> {
        if data.len() < 2 {
            return Err(Error::InsufficientData { needed: 2, remaining: data.len() });
        }

        let found_type = data[0] & 0x7F;
        if ((data[0] & 0x80) == 0) || found_type != id {
            return Err(Error::InvalidTvType(found_type));
        }

        <T as LLRPDecodable>::decode(&data[1..])
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
pub(crate) struct BitContainer {
    pub bits: u32,
    pub valid_bits: u8,
}

impl BitContainer {
    pub(crate) fn read_bits<'a>(
        &mut self,
        mut data: &'a [u8],
        num_bits: u8,
    ) -> Result<(u32, &'a [u8])> {
        while self.valid_bits < num_bits {
            if data.is_empty() {
                let needed_bits = num_bits - self.valid_bits;
                let needed_bytes = (1 + (needed_bits - 1) / 8) as usize;
                return Err(Error::InsufficientData { remaining: 0, needed: needed_bytes });
            }
            self.bits = (self.bits << 8) | data[0].reverse_bits() as u32;
            self.valid_bits += 8;
            data = &data[1..];
        }

        let out = self.bits & ((1 << num_bits) - 1);
        self.bits = self.bits >> num_bits;
        self.valid_bits -= num_bits;

        Ok((out, data))
    }
}
