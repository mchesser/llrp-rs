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
    fn encode(&self, buffer: &mut Vec<u8>);

    fn id(&self) -> u16 {
        Self::ID
    }
}

pub trait TlvParameter: Sized {
    const ID: u16;
}

pub trait LLRPValue: Sized + std::fmt::Debug {
    fn can_decode_type(_: u16) -> bool {
        false
    }

    fn decode(decoder: &mut Decoder) -> Result<Self>;

    fn decode_tv(decoder: &mut Decoder, tv_id: u8) -> Result<Self> {
        decoder.check_param_type(tv_id as u16)?;
        Self::decode(decoder)
    }

    fn encode(&self, _encoder: &mut Encoder) {
        unimplemented!()
    }

    fn encode_tv(&self, encoder: &mut Encoder, tv_id: u8) {
        encoder.write_param_type(ParameterType::Tv(tv_id));
        self.encode(encoder)
    }
}

macro_rules! impl_llrp_value_primitive {
    ($ty: ty) => {
        impl LLRPValue for $ty {
            fn decode(decoder: &mut Decoder) -> Result<Self> {
                let num_bytes = std::mem::size_of::<$ty>();
                Ok(Self::from_be_bytes(decoder.read_bytes(num_bytes)?.try_into().unwrap()))
            }

            fn encode(&self, encoder: &mut Encoder) {
                encoder.write_bytes(&self.to_be_bytes())
            }
        }
    };
}
impl_llrp_value_primitive!(i8);
impl_llrp_value_primitive!(u8);
impl_llrp_value_primitive!(u16);
impl_llrp_value_primitive!(i16);
impl_llrp_value_primitive!(u32);
impl_llrp_value_primitive!(u64);

impl LLRPValue for [u8; 12] {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        Ok(decoder.read_bytes(12)?.try_into().unwrap())
    }

    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_bytes(&self[..])
    }
}

impl LLRPValue for String {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        let len = decoder.read::<u16>()? as usize;
        Ok(String::from_utf8(decoder.read_bytes(len)?.into())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
    }

    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_bytes(&(self.len() as u16).to_be_bytes());
        encoder.write_bytes(self.as_bytes());
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

impl LLRPValue for BitArray {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        let num_bits = decoder.read::<u16>()? as usize;
        Ok(BitArray { bytes: decoder.read_bytes(num_bits / 8)?.into() })
    }

    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_bytes(&((self.bytes.len() / 8) as u16).to_be_bytes());
    }
}

impl<T: LLRPValue> LLRPValue for Option<T> {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        match decoder.peek_param_type() {
            Ok(ty) if T::can_decode_type(ty.as_u16()) => Ok(Some(decoder.read()?)),
            _ => Ok(None),
        }
    }

    fn decode_tv(decoder: &mut Decoder, tv_id: u8) -> Result<Self> {
        match decoder.peek_param_type() {
            Ok(ParameterType::Tv(ty)) if ty == tv_id => Ok(Some(decoder.read_tv(tv_id)?)),
            _ => Ok(None),
        }
    }

    fn encode(&self, encoder: &mut Encoder) {
        if let Some(value) = self {
            value.encode(encoder);
        }
    }

    fn encode_tv(&self, encoder: &mut Encoder, tv_id: u8) {
        if let Some(value) = self {
            value.encode_tv(encoder, tv_id);
        }
    }

    fn can_decode_type(type_num: u16) -> bool {
        T::can_decode_type(type_num)
    }
}

impl<T: LLRPValue> LLRPValue for Box<T> {
    fn decode(decoder: &mut Decoder) -> Result<Self> {
        Ok(Box::new(T::decode(decoder)?))
    }

    fn encode(&self, encoder: &mut Encoder) {
        self.as_ref().encode(encoder)
    }

    fn can_decode_type(type_num: u16) -> bool {
        T::can_decode_type(type_num)
    }
}

impl<T: LLRPValue> LLRPValue for Vec<T> {
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

    fn encode(&self, encoder: &mut Encoder) {
        for value in self {
            value.encode(encoder)
        }
    }

    fn can_decode_type(type_num: u16) -> bool {
        T::can_decode_type(type_num)
    }
}

pub trait Bits {
    fn from_bits(bits: u32) -> Self;
    fn to_bits(&self) -> u32;
}

impl Bits for bool {
    fn from_bits(bits: u32) -> Self {
        (bits & 1) != 0
    }

    fn to_bits(&self) -> u32 {
        match self {
            true => 1,
            false => 0,
        }
    }
}

impl Bits for u8 {
    fn from_bits(bits: u32) -> Self {
        bits as u8
    }

    fn to_bits(&self) -> u32 {
        *self as u32
    }
}

impl Bits for u16 {
    fn from_bits(bits: u32) -> Self {
        bits as u16
    }

    fn to_bits(&self) -> u32 {
        *self as u32
    }
}

pub trait LLRPEnumeration: Sized {
    fn from_value<T: Into<u32>>(value: T) -> Result<Self>;
    fn to_value<T: Bits>(&self) -> T;
}

impl<E: LLRPEnumeration> crate::Bits for E {
    fn from_bits(bits: u32) -> Self {
        Self::from_value(bits).unwrap()
    }

    fn to_bits(&self) -> u32 {
        self.to_value::<u16>() as u32
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
}

impl<'a> Decoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Decoder<'a> {
        Decoder { bytes, bits: 0, valid_bits: 0 }
    }

    pub fn tlv_param<T, F>(&mut self, tlv_id: u16, decode: F) -> Result<T>
    where
        F: FnOnce(&mut Decoder<'a>) -> Result<T>,
    {
        let mut decoder = self.clone();
        decoder.check_param_type(tlv_id)?;

        // Decode the parameter length field.
        // Note: The length field covers the entire parameter including the header
        let param_len = decoder.read::<u16>()? as usize;
        if param_len < 4 || param_len > self.bytes.len() {
            return Err(Error::TlvParameterLengthInvalid(param_len as u16));
        }
        decoder.bytes = &self.bytes[4..param_len];

        let result = decode(&mut decoder)?;
        decoder.validate_consumed()?;

        self.bytes = &self.bytes[param_len..];

        Ok(result)
    }

    pub fn array<T, F>(&mut self, mut decode: F) -> Result<Vec<T>>
    where
        T: LLRPValue,
        F: FnMut(&mut Decoder<'a>) -> Result<T>,
    {
        (0..self.read::<u16>()?).map(|_| decode(self)).collect()
    }

    pub fn read_enum<T, U>(&mut self) -> Result<T>
    where
        T: LLRPEnumeration,
        U: LLRPValue + Into<u32>,
    {
        T::from_value(self.read::<U>()?)
    }

    pub fn read_enum_bits<T>(&mut self, num_bits: u8) -> Result<T>
    where
        T: LLRPEnumeration,
    {
        self.read_bits(num_bits)
    }

    pub fn read_enum_array<T, U>(&mut self) -> Result<Vec<T>>
    where
        T: LLRPEnumeration,
        U: LLRPValue + Into<u32>,
    {
        (0..self.read::<u16>()?).map(|_| T::from_value(self.read::<U>()?)).collect()
    }

    fn check_param_type(&mut self, type_id: u16) -> Result<()> {
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
        if self.bytes.len() < 2 {
            return Err(Error::InsufficientData { needed: 2, remaining: self.bytes.len() });
        }

        if self.bytes[0] & 0b1000_0000 != 0 {
            return Ok(ParameterType::Tv(self.bytes[0] & 0b0111_1111));
        }

        // [6-bit resv, 10-bit message type]
        Ok(ParameterType::Tlv(u16::from_be_bytes([self.bytes[0], self.bytes[1]]) & 0b11_1111_1111))
    }

    pub fn read<T: LLRPValue>(&mut self) -> Result<T> {
        T::decode(self)
    }

    pub fn read_tv<T: LLRPValue>(&mut self, tv_id: u8) -> Result<T> {
        T::decode_tv(self, tv_id)
    }

    pub fn read_bits<T: Bits>(&mut self, num_bits: u8) -> Result<T> {
        while self.valid_bits < num_bits {
            self.bits = (self.bits << 8) | self.read::<u8>()? as u32;
            self.valid_bits += 8;
        }

        let offset = self.valid_bits - num_bits;
        let out = self.bits >> offset;
        self.bits = self.bits & ((1 << offset) - 1);
        self.valid_bits -= num_bits;

        Ok(Bits::from_bits(out))
    }

    pub(crate) fn read_bytes(&mut self, num_bytes: usize) -> Result<&'a [u8]> {
        if self.bytes.len() < num_bytes {
            return Err(Error::InsufficientData { needed: num_bytes, remaining: self.bytes.len() });
        }

        let result = &self.bytes[..num_bytes];
        self.bytes = &self.bytes[num_bytes..];

        Ok(result)
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

pub struct Encoder<'a> {
    buffer: &'a mut Vec<u8>,
    bits: u32,
    valid_bits: u8,
}

impl<'a> Encoder<'a> {
    pub fn new(buffer: &'a mut Vec<u8>) -> Encoder<'a> {
        Encoder { buffer, bits: 0, valid_bits: 0 }
    }

    pub fn tlv_param(&mut self, tlv_id: u16, encode: impl FnOnce(&mut Encoder<'a>)) {
        self.write_param_type(ParameterType::Tlv(tlv_id));

        let offset = self.buffer.len();
        self.write_bytes(&[0, 0]);

        encode(self);

        let param_len = (self.buffer.len() - offset + 2) as u16;
        self.buffer[offset..offset + 2].copy_from_slice(&param_len.to_be_bytes());
    }

    pub fn array<T>(&mut self, items: &[T], mut encode: impl FnMut(&mut Encoder<'a>, &T))
    where
        T: LLRPValue,
    {
        self.write_bytes(&(items.len() as u16).to_be_bytes());
        for item in items {
            encode(self, item)
        }
    }

    pub fn write_enum<T, U>(&mut self, item: &T)
    where
        T: LLRPEnumeration,
        U: LLRPValue + Bits,
    {
        self.write(&item.to_value::<U>());
    }

    pub fn write_enum_bits<T>(&mut self, item: &T, num_bits: u8)
    where
        T: LLRPEnumeration,
    {
        let value = item.to_value::<u16>();
        self.write_bits(&value, num_bits)
    }

    pub fn write_enum_array<T, U>(&mut self, items: &[T])
    where
        T: LLRPEnumeration,
        U: LLRPValue + Bits,
    {
        self.write_bytes(&(items.len() as u16).to_be_bytes());
        for item in items {
            self.write_enum::<T, U>(item)
        }
    }

    fn write_param_type(&mut self, type_num: ParameterType) {
        match type_num {
            ParameterType::Tv(id) => {
                self.write_bytes(&[id | 0b1000_0000]);
            }
            ParameterType::Tlv(id) => {
                self.write_bytes(&id.to_be_bytes());
            }
        }
    }

    pub fn write<T: LLRPValue>(&mut self, value: &T) {
        value.encode(self)
    }

    pub fn write_tv<T: LLRPValue>(&mut self, value: &T, tv_id: u8) {
        value.encode_tv(self, tv_id)
    }

    pub fn write_bits<T: Bits>(&mut self, value: &T, num_bits: u8) {
        let bits = value.to_bits();

        self.bits = (self.bits << num_bits) | bits;
        self.valid_bits += num_bits;

        while self.valid_bits >= 8 {
            self.write_bytes(&[(self.bits & 0xFF) as u8]);
            self.bits = self.bits >> 8;
            self.valid_bits -= 8;
        }
    }

    pub(crate) fn write_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }
}
