use std::io;

use byteorder::{BigEndian, ReadBytesExt};

use llrp_common::LLRPMessage;

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

#[rustfmt::skip]
pub fn deserialize_message(message_type: u16, data: &[u8]) -> io::Result<Message> {
    let message = match message_type {
        GetSupportedVersion::ID => {
            Message::GetSupportedVersion(GetSupportedVersion::decode(data)?.0)
        }
        GetSupportedVersionResponse::ID => {
            Message::GetSupportedVersionResponse(GetSupportedVersionResponse::decode(data)?.0)
        }
        SetProtocolVersion::ID => {
            Message::SetProtocolVersion(SetProtocolVersion::decode(data)?.0)
        }
        SetProtocolVersionResponse::ID => {
            Message::SetProtocolVersionResponse(SetProtocolVersionResponse::decode(data)?.0)
        }
        GetReaderCapabilities::ID => {
            Message::GetReaderCapabilities(GetReaderCapabilities::decode(data)?.0)
        }
        GetReaderCapabilitiesResponse::ID => {
            Message::GetReaderCapabilitiesResponse(GetReaderCapabilitiesResponse::decode(data)?.0)
        }
        AddRoSpec::ID => {
            Message::AddRoSpec(AddRoSpec::decode(data)?.0)
        }
        AddRoSpecResponse::ID => {
            Message::AddRoSpecResponse(AddRoSpecResponse::decode(data)?.0)
        }
        DeleteRoSpec::ID => {
            Message::DeleteRoSpec(DeleteRoSpec::decode(data)?.0)
        }
        DeleteRoSpecResponse::ID => {
            Message::DeleteRoSpecResponse(DeleteRoSpecResponse::decode(data)?.0)
        }
        StartRoSpec::ID => {
            Message::StartRoSpec(StartRoSpec::decode(data)?.0)
        }
        StartRoSpecResponse::ID => {
            Message::StartRoSpecResponse(StartRoSpecResponse::decode(data)?.0)
        }
        StopRoSpec::ID => {
            Message::StopRoSpec(StopRoSpec::decode(data)?.0)
        }
        StopRoSpecResponse::ID => {
            Message::StopRoSpecResponse(StopRoSpecResponse::decode(data)?.0)
        }
        EnableRoSpec::ID => {
            Message::EnableRoSpec(EnableRoSpec::decode(data)?.0)
        }
        EnableRoSpecResponse::ID => {
            Message::EnableRoSpecResponse(EnableRoSpecResponse::decode(data)?.0)
        }
        DisableRoSpec::ID => {
            Message::DisableRoSpec(DisableRoSpec::decode(data)?.0)
        }
        DisableRoSpecResponse::ID => {
            Message::DisableRoSpecResponse(DisableRoSpecResponse::decode(data)?.0)
        }
        GetRoSpecs::ID => {
            Message::GetRoSpecs(GetRoSpecs::decode(data)?.0)
        }
        GetRoSpecsResponse::ID => {
            Message::GetRoSpecsResponse(GetRoSpecsResponse::decode(data)?.0)
        }
        AddAccessSpec::ID => {
            Message::AddAccessSpec(AddAccessSpec::decode(data)?.0)
        }
        AddAccessSpecResponse::ID => {
            Message::AddAccessSpecResponse(AddAccessSpecResponse::decode(data)?.0)
        }
        DeleteAccessSpec::ID => {
            Message::DeleteAccessSpec(DeleteAccessSpec::decode(data)?.0)
        }
        DeleteAccessSpecResponse::ID => {
            Message::DeleteAccessSpecResponse(DeleteAccessSpecResponse::decode(data)?.0)
        }
        EnableAccessSpec::ID => {
            Message::EnableAccessSpec(EnableAccessSpec::decode(data)?.0)
        }
        EnableAccessSpecResponse::ID => {
            Message::EnableAccessSpecResponse(EnableAccessSpecResponse::decode(data)?.0)
        }
        DisableAccessSpec::ID => {
            Message::DisableAccessSpec(DisableAccessSpec::decode(data)?.0)
        }
        DisableAccessSpecResponse::ID => {
            Message::DisableAccessSpecResponse(DisableAccessSpecResponse::decode(data)?.0)
        }
        GetAccessSpecs::ID => {
            Message::GetAccessSpecs(GetAccessSpecs::decode(data)?.0)
        }
        GetAccessSpecsResponse::ID => {
            Message::GetAccessSpecsResponse(GetAccessSpecsResponse::decode(data)?.0)
        }
        ClientRequestOp::ID => {
            Message::ClientRequestOp(ClientRequestOp::decode(data)?.0)
        }
        ClientRequestOpResponse::ID => {
            Message::ClientRequestOpResponse(ClientRequestOpResponse::decode(data)?.0)
        }
        GetReaderConfig::ID => {
            Message::GetReaderConfig(GetReaderConfig::decode(data)?.0)
        }
        GetReaderConfigResponse::ID => {
            Message::GetReaderConfigResponse(GetReaderConfigResponse::decode(data)?.0)
        }
        SetReaderConfig::ID => {
            Message::SetReaderConfig(SetReaderConfig::decode(data)?.0)
        }
        SetReaderConfigResponse::ID => {
            Message::SetReaderConfigResponse(SetReaderConfigResponse::decode(data)?.0)
        }
        CloseConnection::ID => {
            Message::CloseConnection(CloseConnection::decode(data)?.0)
        }
        CloseConnectionResponse::ID => {
            Message::CloseConnectionResponse(CloseConnectionResponse::decode(data)?.0)
        }
        GetReport::ID => {
            Message::GetReport(GetReport::decode(data)?.0)
        }
        RoAccessReport::ID => {
            Message::RoAccessReport(RoAccessReport::decode(data)?.0)
        }
        KeepAlive::ID => {
            Message::KeepAlive(KeepAlive::decode(data)?.0)
        }
        KeepAliveAck::ID => {
            Message::KeepAliveAck(KeepAliveAck::decode(data)?.0)
        }
        ReaderEventNotification::ID => {
            Message::ReaderEventNotification(ReaderEventNotification::decode(data)?.0)
        }
        EnableEventsAndReports::ID => {
            Message::EnableEventsAndReports(EnableEventsAndReports::decode(data)?.0)
        }
        CustomMessage::ID => {
            Message::CustomMessage(CustomMessage::decode(data)?.0)
        }
        ErrorMessage::ID => {
            Message::ErrorMessage(ErrorMessage::decode(data)?.0)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid message type: {}", message_type),
            ))
        }
    };

    Ok(message)
}
