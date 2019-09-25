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

pub trait LLRPMessage: Sized {
    const ID: u16;

    fn decode(data: &[u8]) -> Result<(Self, &[u8])>;

    fn id(&self) -> u16 {
        Self::ID
    }
}

pub trait LLRPDecodable: Sized + std::fmt::Debug {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])>;

    fn decode_tv(data: &[u8], tv_id: u8) -> Result<(Self, &[u8])> {
        if data.len() < 2 {
            return Err(Error::InsufficientData { needed: 2, remaining: data.len() });
        }

        let found_type = data[0] & 0b0111_1111;
        if ((data[0] & 0b1000_0000) == 0) || found_type != tv_id {
            return Err(Error::InvalidTvType(found_type));
        }

        Self::decode(&data[1..])
    }

    fn can_decode_type(_: u16) -> bool {
        false
    }
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

    fn decode_tlv(_data: &[u8]) -> Result<(Self, &[u8])> {
        unimplemented!()
    }

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

#[derive(Default, Clone)]
pub(crate) struct Decoder<'a> {
    bytes: &'a [u8],
    bits: u32,
    valid_bits: u8,
    level: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Decoder<'a> {
        Decoder { bytes, bits: 0, valid_bits: 0, level: 0 }
    }

    pub fn tlv_param_decoder(&mut self, tlv_id: u16) -> Result<Decoder<'a>> {
        let mut decoder = self.clone();
        decoder.level += 1;

        let type_ = decoder.read_tlv_header()?;
        if type_ != tlv_id {
            return Err(Error::InvalidType(type_));
        }

        // Decode the parameter length field.
        // Note: The length field covers the entire parameter including the header
        let param_len = decoder.read::<u16>()? as usize;
        if param_len < 4 || param_len > self.bytes.len() {
            return Err(Error::TlvParameterLengthInvalid(param_len as u16));
        }

        decoder.bytes = &self.bytes[4..param_len];
        self.bytes = &self.bytes[param_len..];

        Ok(decoder)
    }

    pub(crate) fn read_tv<T: LLRPDecodable>(&mut self, tv_id: u8) -> Result<T> {
        eprintln!("{:width$} reading tv: {} (id = {})", "", std::any::type_name::<T>(), tv_id, width = self.level * 4);
        let (value, remaining) = <T as LLRPDecodable>::decode_tv(&self.bytes, tv_id)?;
        eprintln!("{:width$} got: {:?}", "", value, width = self.level * 4);
        self.bytes = remaining;
        Ok(value)
    }

    pub fn read_tlv_header(&mut self) -> Result<u16> {
        // The first two bytes consist of [6-bit resv, 10-bit message type], so we read 2 bytes and
        // mask the message type
        Ok(self.read::<u16>()? & 0b11_1111_1111)
    }

    pub fn read<T: LLRPDecodable>(&mut self) -> Result<T> {
        eprintln!("{:width$} reading: {}", "", std::any::type_name::<T>(), width = self.level * 4);
        let (value, remaining) = T::decode(self.bytes)?;
        eprintln!("{:width$} got: {:?}", "", value, width = self.level * 4);

        self.bytes = remaining;
        Ok(value)
    }

    pub fn read_bits(&mut self, num_bits: u8) -> Result<u32> {
        while self.valid_bits < num_bits {
            self.bits = (self.bits << 8) | self.read::<u8>()? as u32;
            self.valid_bits += 8;
        }

        let offset = self.valid_bits - num_bits;
        let out = self.bits >> offset;
        self.bits = self.bits & ((1 << offset) - 1);
        self.valid_bits -= num_bits;

        Ok(out)
    }

    /// Ensures that all bytes were consumed when parsing the struct fields
    /// TODO: consider adding a feature to run in `relaxed` mode where this error is ignored
    pub(crate) fn validate_consumed(&self) -> Result<()> {
        if !self.bytes.is_empty() {
            return Err(Error::TrailingBytes(self.bytes.len()));
        }
        Ok(())
    }
}

impl<T: LLRPDecodable> LLRPDecodable for Option<T> {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() == 0 {
            return Ok((None, data));
        }

        match get_message_type(data) {
            Ok(ty) if T::can_decode_type(ty) => {
                let (field, rest) = T::decode(data)?;
                Ok((Some(field), rest))
            }
            _ => Ok((None, data)),
        }
    }

    fn decode_tv(data: &[u8], tv_id: u8) -> Result<(Self, &[u8])> {
        if data.len() == 0 {
            return Ok((None, data));
        }

        match get_message_type(data) {
            Ok(ty) if ty == tv_id as u16 => {
                let (field, rest) = T::decode_tv(data, tv_id)?;
                Ok((Some(field), rest))
            }
            _ => Ok((None, data)),
        }
    }

    fn can_decode_type(type_num: u16) -> bool {
        T::can_decode_type(type_num)
    }
}

impl<T: LLRPDecodable> LLRPDecodable for Box<T> {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        let (result, rest) = T::decode(data)?;
        Ok((Box::new(result), rest))
    }

    fn can_decode_type(type_num: u16) -> bool {
        T::can_decode_type(type_num)
    }
}

impl<T: LLRPDecodable> LLRPDecodable for Vec<T> {
    fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
        let mut output = vec![];

        let mut rest = data;
        while rest.len() > 0 {
            match get_message_type(rest) {
                Ok(ty) if T::can_decode_type(ty) => {
                    let (field, new_rest) = T::decode(rest)?;
                    output.push(field);
                    rest = new_rest;
                }
                _ => break,
            }
        }

        Ok((output, rest))
    }

    fn can_decode_type(type_num: u16) -> bool {
        T::can_decode_type(type_num)
    }
}

pub fn get_message_type(data: &[u8]) -> Result<u16> {
    if data.len() < 1 {
        return Err(Error::InsufficientData { needed: 1, remaining: data.len() });
    }

    if data[0] & 0b1000_0000 != 0 {
        // This is a tv parameter
        let tv_param = (data[0] & 0b0111_1111) as u16;
        return Ok(tv_param);
    }

    if data.len() < 2 {
        return Err(Error::InsufficientData { needed: 2, remaining: data.len() });
    }

    // This is a tlv parameter [6-bit resv, 10-bit message type]
    let tlv_param = u16::from_be_bytes([data[0], data[1]]) & 0b11_1111_1111;
    Ok(tlv_param)
}
