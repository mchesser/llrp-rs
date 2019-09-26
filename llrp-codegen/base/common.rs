use std::{convert::TryInto, fmt, io};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InsufficientData { needed: usize, remaining: usize },
    TrailingBits(usize),
    TrailingBytes(usize),
    TlvParameterLengthInvalid(u16),
    InvalidType(u16),
    InvalidVariant(u32),
    UnknownMessageId(u32),
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
            Error::InvalidVariant(value) => write!(f, "Invalid variant: {}", value),
            Error::UnknownMessageId(id) => write!(f, "Unknown message id: {}", id),
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

pub trait LLRPMessage: Sized {
    const ID: u16;

    fn decode(data: &[u8]) -> Result<(Self, &[u8])>;

    fn id(&self) -> u16 {
        Self::ID
    }
}

pub trait TlvParameter: Sized {
    const ID: u16;
}

pub trait LLRPDecodable: Sized + std::fmt::Debug {
    fn decode(decoder: &mut Decoder) -> Result<Self>;

    fn decode_tv(decoder: &mut Decoder, tv_id: u8) -> Result<Self> {
        let mut tv_decoder = decoder.tv_param_decoder(tv_id)?;
        let result = tv_decoder.read()?;
        decoder.bytes = tv_decoder.bytes;
        Ok(result)
    }

    fn can_decode_type(_: u16) -> bool {
        false
    }
}

impl LLRPDecodable for bool {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        Ok(decoder.read_bytes(1)?[0] != 0)
    }
}

macro_rules! impl_llrp_decodable_primitive {
    ($ty: ty) => {
        impl LLRPDecodable for $ty {
            fn decode(decoder: &mut Decoder) -> Result<Self> {
                let num_bytes = std::mem::size_of::<$ty>();
                Ok(Self::from_be_bytes(decoder.read_bytes(num_bytes)?.try_into().unwrap()))
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
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        Ok(decoder.read_bytes(12)?.try_into().unwrap())
    }
}

impl LLRPDecodable for String {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        let len = decoder.read::<u16>()? as usize;
        Ok(String::from_utf8(decoder.read_bytes(len)?.into())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
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
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        let num_bits = decoder.read::<u16>()? as usize;
        Ok(BitArray { bytes: decoder.read_bytes(num_bits / 8)?.into() })
    }
}

impl<T: LLRPDecodable> LLRPDecodable for Option<T> {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        match decoder.peek_param_type() {
            Ok(ty) if T::can_decode_type(ty.as_u16()) => Ok(Some(T::decode(decoder)?)),
            _ => Ok(None),
        }
    }

    fn decode_tv(decoder: &mut Decoder, tv_id: u8) -> Result<Self> {
        match decoder.peek_param_type() {
            Ok(ParameterType::Tv(ty)) if ty == tv_id => Ok(Some(T::decode_tv(decoder, tv_id)?)),
            _ => Ok(None),
        }
    }

    fn can_decode_type(type_num: u16) -> bool {
        T::can_decode_type(type_num)
    }
}

impl<T: LLRPDecodable> LLRPDecodable for Box<T> {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        Ok(Box::new(T::decode(decoder)?))
    }

    fn can_decode_type(type_num: u16) -> bool {
        T::can_decode_type(type_num)
    }
}

impl<T: LLRPDecodable> LLRPDecodable for Vec<T> {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        let mut output = vec![];

        loop {
            match decoder.get_message_type() {
                Ok(ty) if T::can_decode_type(ty) => output.push(T::decode(decoder)?),
                _ => break,
            }
        }

        Ok(output)
    }

    fn can_decode_type(type_num: u16) -> bool {
        T::can_decode_type(type_num)
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

pub enum ParameterType {
    Tv(u8),
    Tlv(u16),
}

impl ParameterType {
    fn as_u16(&self) -> u16 {
        match *self {
            ParameterType::Tv(id) => id as u16,
            ParameterType::Tlv(id) => id,
        }
    }
}

#[derive(Default, Clone)]
pub struct Decoder<'a> {
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
        decoder.check_message_type(tlv_id)?;

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

    pub fn tv_param_decoder(&mut self, tv_id: u8) -> Result<Decoder<'a>> {
        let mut decoder = self.clone();
        decoder.level += 1;
        decoder.check_message_type(tv_id as u16)?;
        Ok(decoder)
    }

    pub(crate) fn read_tv<T: LLRPDecodable>(&mut self, tv_id: u8) -> Result<T> {
        T::decode_tv(self, tv_id)
    }

    fn check_message_type(&mut self, type_id: u16) -> Result<()> {
        match self.peek_param_type()? {
            ParameterType::Tv(id) if id as u16 == type_id => {
                self.bytes = &self.bytes[1..];
                Ok(())
            }
            ParameterType::Tlv(id) if id == type_id => {
                self.bytes = &self.bytes[2..];
                Ok(())
            }
            other => Err(Error::InvalidType(other.as_u16())),
        }
    }

    pub fn get_message_type(&self) -> Result<u16> {
        match self.peek_param_type()? {
            ParameterType::Tv(id) => Ok(id as u16),
            ParameterType::Tlv(id) => Ok(id),
        }
    }

    pub fn peek_param_type(&self) -> Result<ParameterType> {
        let data = self.bytes;

        if data.len() < 2 {
            return Err(Error::InsufficientData { needed: 2, remaining: data.len() });
        }

        if data[0] & 0b1000_0000 != 0 {
            return Ok(ParameterType::Tv(data[0] & 0b0111_1111));
        }

        // [6-bit resv, 10-bit message type]
        Ok(ParameterType::Tlv(u16::from_be_bytes([data[0], data[1]]) & 0b11_1111_1111))
    }

    pub fn read<T: LLRPDecodable>(&mut self) -> Result<T> {
        T::decode(self)
    }

    pub fn read_from_bits<T: FromBits>(&mut self, num_bits: u8) -> Result<T> {
        Ok(FromBits::from_bits(self.read_bits(num_bits)?))
    }

    pub(crate) fn read_bytes(&mut self, num_bytes: usize) -> Result<&'a [u8]> {
        if self.bytes.len() < num_bytes {
            return Err(Error::InsufficientData { needed: num_bytes, remaining: self.bytes.len() });
        }

        let result = &self.bytes[..num_bytes];
        self.bytes = &self.bytes[num_bytes..];

        Ok(result)
    }

    pub(crate) fn read_bits(&mut self, num_bits: u8) -> Result<u32> {
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
