use std::io;

use byteorder::{BigEndian, ReadBytesExt};

pub struct BinaryMessage {
    pub ver: u8,
    pub message_type: u16,
    pub id: u32,
    pub value: Vec<u8>,
}

pub fn read_message<R: io::Read>(mut reader: R) -> io::Result<BinaryMessage> {
    const HEADER_LENGTH: usize = 10;

    // First 16 bits are packed with [3-bit reserved, 3-bit version, 10-bit message type]
    let prefix = reader.read_u16::<BigEndian>()?;
    let ver = ((prefix >> 10) & 0b111) as u8;
    let message_type = prefix & 0b11_1111_1111;

    let length = reader.read_u32::<BigEndian>()? as usize;
    if length < HEADER_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid length: {}", length),
        ));
    }

    let id = reader.read_u32::<BigEndian>()?;

    let mut value = vec![0; length - HEADER_LENGTH];
    reader.read_exact(&mut value)?;

    Ok(BinaryMessage { ver, message_type, id, value })
}
