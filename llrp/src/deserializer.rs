use std::io;

use byteorder::{BigEndian, ReadBytesExt};

use llrp_common::DecodableMessage;

use crate::messages::*;

pub struct BinaryMessage {
    pub ver: u8,
    pub message_type: u16,
    pub id: u32,
    pub value: Vec<u8>,
}

pub fn deserialize_raw<R: io::Read>(mut reader: R) -> io::Result<BinaryMessage> {
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

    Ok(BinaryMessage {
        ver,
        message_type,
        id,
        value,
    })
}

pub fn deserialize_message(message_type: u16, data: &[u8]) -> io::Result<Box<dyn std::any::Any>> {
    let message = match message_type {
        GetSupportedVersion::ID => {
            Box::new(GetSupportedVersion::decode(data)?) as Box<dyn std::any::Any>
        }
        GetSupportedVersionResponse::ID => {
            Box::new(GetSupportedVersionResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        SetProtocolVersion::ID => {
            Box::new(SetProtocolVersion::decode(data)?) as Box<dyn std::any::Any>
        }
        SetProtocolVersionResponse::ID => {
            Box::new(SetProtocolVersionResponse::decode(data)?) as Box<dyn std::any::Any>
        }

        GetReaderCapabilities::ID => {
            Box::new(GetReaderCapabilities::decode(data)?) as Box<dyn std::any::Any>
        }
        GetReaderCapabilitiesResponse::ID => {
            Box::new(GetReaderCapabilitiesResponse::decode(data)?) as Box<dyn std::any::Any>
        }

        AddRoSpec::ID => Box::new(AddRoSpec::decode(data)?) as Box<dyn std::any::Any>,
        AddRoSpecResponse::ID => {
            Box::new(AddRoSpecResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        DeleteRoSpec::ID => Box::new(DeleteRoSpec::decode(data)?) as Box<dyn std::any::Any>,
        DeleteRoSpecResponse::ID => {
            Box::new(DeleteRoSpecResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        StartRoSpec::ID => Box::new(StartRoSpec::decode(data)?) as Box<dyn std::any::Any>,
        StartRoSpecResponse::ID => {
            Box::new(StartRoSpecResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        StopRoSpec::ID => Box::new(StopRoSpec::decode(data)?) as Box<dyn std::any::Any>,
        StopRoSpecResponse::ID => {
            Box::new(StopRoSpecResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        EnableRoSpec::ID => Box::new(EnableRoSpec::decode(data)?) as Box<dyn std::any::Any>,
        EnableRoSpecResponse::ID => {
            Box::new(EnableRoSpecResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        DisableRoSpec::ID => Box::new(DisableRoSpec::decode(data)?) as Box<dyn std::any::Any>,
        DisableRoSpecResponse::ID => {
            Box::new(DisableRoSpecResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        GetRoSpecs::ID => Box::new(GetRoSpecs::decode(data)?) as Box<dyn std::any::Any>,
        GetRoSpecsResponse::ID => {
            Box::new(GetRoSpecsResponse::decode(data)?) as Box<dyn std::any::Any>
        }

        AddAccessSpec::ID => Box::new(AddAccessSpec::decode(data)?) as Box<dyn std::any::Any>,
        AddAccessSpecResponse::ID => {
            Box::new(AddAccessSpecResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        DeleteAccessSpec::ID => Box::new(DeleteAccessSpec::decode(data)?) as Box<dyn std::any::Any>,
        DeleteAccessSpecResponse::ID => {
            Box::new(DeleteAccessSpecResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        EnableAccessSpec::ID => Box::new(EnableAccessSpec::decode(data)?) as Box<dyn std::any::Any>,
        EnableAccessSpecResponse::ID => {
            Box::new(EnableAccessSpecResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        DisableAccessSpec::ID => {
            Box::new(DisableAccessSpec::decode(data)?) as Box<dyn std::any::Any>
        }
        DisableAccessSpecResponse::ID => {
            Box::new(DisableAccessSpecResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        GetAccessSpecs::ID => Box::new(GetAccessSpecs::decode(data)?) as Box<dyn std::any::Any>,
        GetAccessSpecsResponse::ID => {
            Box::new(GetAccessSpecsResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        ClientRequestOp::ID => Box::new(ClientRequestOp::decode(data)?) as Box<dyn std::any::Any>,
        ClientRequestOpResponse::ID => {
            Box::new(ClientRequestOpResponse::decode(data)?) as Box<dyn std::any::Any>
        }

        GetReaderConfig::ID => Box::new(GetReaderConfig::decode(data)?) as Box<dyn std::any::Any>,
        GetReaderConfigResponse::ID => {
            Box::new(GetReaderConfigResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        SetReaderConfig::ID => Box::new(SetReaderConfig::decode(data)?) as Box<dyn std::any::Any>,
        SetReaderConfigResponse::ID => {
            Box::new(SetReaderConfigResponse::decode(data)?) as Box<dyn std::any::Any>
        }
        CloseConnection::ID => Box::new(CloseConnection::decode(data)?) as Box<dyn std::any::Any>,
        CloseConnectionResponse::ID => {
            Box::new(CloseConnectionResponse::decode(data)?) as Box<dyn std::any::Any>
        }

        GetReport::ID => Box::new(GetReport::decode(data)?) as Box<dyn std::any::Any>,
        RoAccessReport::ID => Box::new(RoAccessReport::decode(data)?) as Box<dyn std::any::Any>,
        ReaderEventNotification::ID => {
            Box::new(ReaderEventNotification::decode(data)?) as Box<dyn std::any::Any>
        }
        KeepAlive::ID => Box::new(KeepAlive::decode(data)?) as Box<dyn std::any::Any>,
        KeepAliveAck::ID => Box::new(KeepAliveAck::decode(data)?) as Box<dyn std::any::Any>,
        EnableEventsAndReports::ID => {
            Box::new(EnableEventsAndReports::decode(data)?) as Box<dyn std::any::Any>
        }

        CustomMessage::ID => Box::new(CustomMessage::decode(data)?) as Box<dyn std::any::Any>,

        ErrorMessage::ID => Box::new(ErrorMessage::decode(data)?) as Box<dyn std::any::Any>,

        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid message type: {}", message_type),
            ))
        }
    };

    Ok(message)
}
