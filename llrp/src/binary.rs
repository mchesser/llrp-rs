use std::io;

use byteorder::{BigEndian, ReadBytesExt};

use crate::{messages::Message, LLRPMessage};

#[derive(Debug, Clone)]
pub struct BinaryMessage {
    pub ver: u8,
    pub message_type: u16,
    pub id: u32,
    pub value: Vec<u8>,
}

impl BinaryMessage {
    pub fn from_message<T: LLRPMessage>(id: u32, message: T) -> crate::Result<BinaryMessage> {
        let mut buffer = vec![];
        message.encode(&mut buffer);
        Ok(BinaryMessage { ver: 1, message_type: T::ID, id, value: buffer })
    }

    pub fn to_message<T: LLRPMessage>(&self) -> crate::Result<T> {
        let (msg, _) = T::decode(&self.value)?;
        Ok(msg)
    }

    pub fn from_dynamic_message(id: u32, message: &Message) -> crate::Result<BinaryMessage> {
        let mut buffer = vec![];
        message.encode(&mut buffer);
        Ok(BinaryMessage { ver: 1, message_type: message.message_type(), id, value: buffer })
    }

    pub fn to_dynamic_message(&self) -> crate::Result<Message> {
        Message::decode(self.message_type as u32, &self.value)
    }
}

const LLRP_HEADER_LENGTH: usize = 10;

pub fn read_message<R: io::Read>(mut reader: R) -> io::Result<BinaryMessage> {
    // First 16 bits are packed with [3-bit reserved, 3-bit version, 10-bit message type]
    let prefix = reader.read_u16::<BigEndian>()?;
    let ver = ((prefix >> 10) & 0b111) as u8;
    let message_type = prefix & 0b11_1111_1111;

    let length = reader.read_u32::<BigEndian>()? as usize;
    if length < LLRP_HEADER_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid length: {}", length),
        ));
    }

    let id = reader.read_u32::<BigEndian>()?;

    let mut value = vec![0; length - LLRP_HEADER_LENGTH];
    reader.read_exact(&mut value)?;

    Ok(BinaryMessage { ver, message_type, id, value })
}

pub fn write_message<W: io::Write>(mut writer: W, message: BinaryMessage) -> io::Result<()> {
    let prefix = [
        ((message.ver & 0b111) << 2) | (message.message_type >> 8) as u8,
        message.message_type as u8,
    ];

    writer.write_all(&prefix)?;
    writer.write_all(&((message.value.len() + LLRP_HEADER_LENGTH) as u32).to_be_bytes())?;
    writer.write_all(&message.id.to_be_bytes())?;
    writer.write_all(&message.value)
}
