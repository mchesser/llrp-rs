use llrp_common::{BitArray, TvDecodable};
use llrp_message::{llrp_parameter, TryFromU16};

use std::{convert::TryFrom, convert::TryInto, io};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, TryFromU16)]
pub enum StatusCode {
    /// The message was recieved and processed successfully
    M_Success = 0,

    /// An error occured with a parameter of this message
    M_ParameterError = 100,

    /// An error occured with a parameter of this message
    M_FieldError = 101,

    /// Unexpected parameter was missing from this message
    M_UnexpectedParameter = 102,

    /// A required message was missing from this message
    M_MissingParameter = 103,

    /// Indicate that a parameter, for which there must only be one instance at the Reader, was
    /// seen more than once in this message.
    M_DuplicateParameter = 104,

    /// The maximum number of instances of the parameter has been exceeded at the Reader.
    M_OverflowParameter = 105,

    /// The maximum number of instances of the field has been exceeded at the Reader.
    M_OverflowField = 106,

    /// An unknown parameter was received in the message.
    M_UnknownParameter = 107,

    /// The field is unknown or not found at the Reader.
    M_UnknownField = 108,

    /// An unsupported message type was received.
    M_UnsupportedMessage = 109,

    /// The LLRP version in the received message is not supported by the Reader.
    M_UnsupportedVersion = 110,

    /// The Parameter in the received message is not supported by the Reader.
    M_UnsupportedParameter = 111,

    /// The message received was unexpected by the Reader.
    M_UnexpectedMessage = 112,

    /// An error occurred with a parameter of this parameter.
    P_ParameterErrorParameter = 200,

    /// An error occurred with a field of this parameter.
    P_FieldError = 201,

    /// An unexpected parameter was received with this message.
    P_UnexpectedParameter = 202,

    /// A required parameter was missing from this parameter.
    P_MissingParameter = 203,

    /// A parameter, for which there must only be one instance, was seen more than once in this
    /// parameter.
    P_DuplicateParameter = 204,

    /// The maximum number of instances of the parameter has been exceeded at the Reader.
    P_OverflowParameter = 205,

    /// The maximum number of instances of the field has been exceeded at the Reader.
    P_OverflowField = 206,

    /// An unknown parameter was received with this message.
    P_UnknownParameter = 207,

    /// The field is unknown or not found at the Reader.
    P_UnknownField = 208,

    /// An unsupported parameter was received.
    P_UnsupportedParameter = 209,

    /// The field value was considered invalid for a non specific reason
    A_InvalidField = 300,

    /// The field value did not fall within an acceptable range
    A_OutOfRange = 301,

    /// There is a problem on the reader
    R_DeviceError = 401,
}

impl llrp_common::LLRPDecodable for StatusCode {
    fn decode(data: &[u8]) -> llrp_common::Result<(Self, &[u8])> {
        if data.len() < 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }
        let value = u16::from_be_bytes(data[..2].try_into().unwrap());

        let status = StatusCode::try_from(value).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid variant: {}", e))
        })?;

        Ok((status, &data[2..]))
    }
}

#[llrp_parameter(id = 287)]
pub struct LLRPStatus {
    pub status_code: StatusCode,
    pub error_description: String,
    pub field_error: Option<FieldError>,
    pub parameter_error: Option<ParameterError>,
}

#[llrp_parameter(id = 288)]
pub struct FieldError {
    pub field_number: u16,
    pub error_code: StatusCode,
}

#[llrp_parameter(id = 289)]
pub struct ParameterError {
    pub field_error: Option<FieldError>,
    pub parameter_error: Option<Box<ParameterError>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CustomParameter;
impl llrp_common::TlvDecodable for CustomParameter {
    fn decode_tlv(_data: &[u8]) -> llrp_common::Result<(Self, &[u8])> {
        Err(llrp_common::Error::InvalidType(0))
    }
}

pub enum ReaderCapabilitiesRequestedData {
    All,
    GeneralDeviceCapabilities,
    LLRPCapabilities,
    RegulatoryCapabilities,
    AirProtocolLLRPCapabilities,
}
impl llrp_common::TlvDecodable for ReaderCapabilitiesRequestedData {}

pub struct GeneralDeviceCapabilities {
    /// The IANA Private Enterprise Number (PEN)
    pub device_manufacturer_name: u32,

    /// Model name
    pub model_name: u32,

    /// UTF-8 string representation of the firmware version
    pub firmware: String,

    /// The maximum number of supported antenntas
    pub max_antennas: u16,

    /// If set to true, the client can set antenna properties, else Client can not set it but only,
    /// query it using `GET_READER_CONFIG`
    pub can_set_antenna_properties: bool,

    /// The maximum receive sensitivity supported by the device. The value is in absolute dBm
    pub max_receive_sensitivity: Option<i16>,

    /// Specifies a table of sensitivity values relative to `max_receive_sensitivity`
    pub receive_sensitivity_table: Vec<ReceiveSensitivityTableEntry>,

    /// Specifies the receive sensitivity range for each of the antennas
    pub per_antenna_receive_sensitivity_range: Vec<PerAntennaReceiveSensitivityRange>,

    /// Speficies the air protocol support for each of the antennas
    pub per_antenna_air_protocol_support: Vec<PerAntennaAirProtocolSupport>,

    /// Describes the GPIO cababilties of the Reader
    pub gpio_support: GpioCapabilities,

    /// If set to tru, the Reader reports time based on UTC timestamps in its reports, else, the
    /// Reader reports time based on Uptime in its report
    pub has_utc_clock_capability: bool,
}
impl llrp_common::TlvDecodable for GeneralDeviceCapabilities {}

pub struct ReceiveSensitivityTableEntry {
    /// The index of the entry
    pub index: u16,

    /// The receive sensitivity value in dB relative to the maximum sensitivity.
    /// Possible values: 0 to 128
    pub receive_sensitivity_value: i32,
}
impl llrp_common::TlvDecodable for ReceiveSensitivityTableEntry {}

pub struct PerAntennaReceiveSensitivityRange {
    /// Antenna id (1-indexed). Possible values: 1 to N where N is the maximum number of antennas
    /// supported by the device
    pub antenna_id: u16,

    /// Specifies the (0-indexed) entry in the `receive_sensitivity_table` for the minimum recieve
    /// sensitivity for this antenna
    pub receive_sensitivity_index_min: u16,

    /// Specifies the (0-indexed) entry in the `receive_sensitivity_table` for the maximum recieve
    /// sensitivity for this antenna
    pub receive_sensitivity_index_max: u16,
}
impl llrp_common::TlvDecodable for PerAntennaReceiveSensitivityRange {}

pub struct PerAntennaAirProtocolSupport {
    /// Antenna id (1-indexed). Possible values: 1 to N where N is the maximum number of antennas
    /// supported by the device
    pub antenna_id: u16,

    /// List of supported protocol IDs
    pub air_protocols_supported: Vec<AirProtocol>,
}
impl llrp_common::TlvDecodable for PerAntennaAirProtocolSupport {}

pub struct GpioCapabilities {
    /// Number of general purpose inputs supported by the device
    pub num_gpis: u16,

    /// Number of general purpose outputs supported by the device
    pub num_gpos: u16,
}
impl llrp_common::TlvDecodable for GpioCapabilities {}

pub struct LLRPCapabilities;
impl llrp_common::TlvDecodable for LLRPCapabilities {}

pub struct RegulatoryCapabilities;
impl llrp_common::TlvDecodable for RegulatoryCapabilities {}

pub struct AirProtocolLLRPCapabilities;
impl llrp_common::TlvDecodable for AirProtocolLLRPCapabilities {}

#[llrp_parameter(id = 177)]
pub struct RoSpec {
    pub id: u32,
    pub priority: u8,
    pub current_state: u8,
    pub boundary_spec: RoBoundarySpec,
    pub spec_list: Vec<AiSpec>,
    pub report_spec: Option<RoReportSpec>,
}

#[llrp_parameter(id = 178)]
pub struct RoBoundarySpec {
    pub start_trigger: RoSpecStartTrigger,
    pub stop_trigger: RoSpecStopTrigger,
}

#[llrp_parameter(id = 179)]
pub struct RoSpecStartTrigger {
    pub trigger_type: u8,

    // FIXME: required when trigger_type == 2
    pub periodic_trigger_value: Option<PeriodicTriggerValue>,

    // FIXME: required when trigger_type == 3
    pub gpi_trigger_value: Option<GPITriggerValue>,
}

#[llrp_parameter(id = 180)]
pub struct PeriodicTriggerValue {
    pub offset: u64,
    pub period: u64,
    pub utc_time: Option<UTCTimestamp>,
}

#[llrp_parameter(id = 181)]
pub struct GPITriggerValue {
    pub gpi_port_num: u16,
    pub gpi_event: bool,
    pub timeout: u64,
}

#[llrp_parameter(id = 182)]
pub struct RoSpecStopTrigger {
    pub trigger_type: u8,
    pub duration_trigger_value: u32,
    pub gpi_trigger_value: Option<GPITriggerValue>,
}

#[llrp_parameter(id = 183)]
pub struct AiSpec {
    #[has_length]
    pub antenna_ids: Vec<u16>,

    pub stop_trigger: AiSpecStopTrigger,
    pub inventory_specs: Vec<InventorySpec>,
    pub custom: Vec<CustomParameter>,
}

#[llrp_parameter(id = 184)]
pub struct AiSpecStopTrigger {
    pub trigger_type: u8,
    pub duration_trigger_value: u32,
    pub gpi_trigger_value: Option<GPITriggerValue>,
    pub tag_observation_trigger_value: Option<TagObservationTriggerValue>,
}

#[llrp_parameter(id = 185)]
pub struct TagObservationTriggerValue {
    pub trigger_type: u8,
    pub _reserved: u8,
    pub number_of_tags: u16,
    pub number_of_attempts: u16,
    pub t: u16,
    pub timeout: u64,
}

#[llrp_parameter(id = 186)]
pub struct InventorySpec {
    pub spec_id: u16,
    pub protocol_id: u8,
    pub antenna_configuration: Vec<AntennaConfiguration>,
    pub custom: Vec<CustomParameter>,
}

#[llrp_parameter(id = 207)]
#[derive(Debug, Eq, PartialEq)]
pub struct AccessSpec {
    pub id: u32,
    pub antenna_id: u16,
    pub protocol_id: u8,
    pub current_state: bool,
    pub ro_spec_id: u32,
    pub stop_trigger: AccessSpecStopTrigger,
    pub command: AccessCommand,
    pub report_spec: Option<AccessReportSpec>,
    pub custom: Vec<CustomParameter>,
}

#[llrp_parameter(id = 208)]
#[derive(Debug, Eq, PartialEq)]
pub struct AccessSpecStopTrigger {
    pub trigger_type: u8,
    pub operation_count: u16,
}

#[llrp_parameter(id = 209)]
#[derive(Debug, Eq, PartialEq)]
pub struct AccessCommand {
    pub tag_spec: C1G2TagSpec,
    pub op_spec: Vec<C1G2BlockWrite>,
    pub custom: Vec<CustomParameter>,
}

#[llrp_parameter(id = 239)]
#[derive(Debug, Eq, PartialEq)]
pub struct AccessReportSpec {
    pub trigger: u8,
}

#[llrp_parameter(id = 240)]
#[derive(Debug, Eq, PartialEq)]
pub struct TagReportData {
    pub epc_data: EpcDataParameter,

    #[tv_param = 9]
    pub ro_spec_id: Option<u16>,

    #[tv_param = 14]
    pub spec_index: Option<u16>,

    #[tv_param = 10]
    pub inventory_param_spec_id: Option<u16>,

    #[tv_param = 1]
    pub antenna_id: Option<u16>,

    #[tv_param = 6]
    pub peak_rssi: Option<u8>,

    #[tv_param = 7]
    pub channel_index: Option<u16>,

    #[tv_param = 2]
    pub first_seen_timestamp_utc: Option<u64>,

    #[tv_param = 3]
    pub first_seen_timestamp_uptime: Option<u64>,

    #[tv_param = 4]
    pub last_seen_timestamp_utc: Option<u64>,

    #[tv_param = 5]
    pub last_seen_timestamp_uptime: Option<u64>,

    #[tv_param = 8]
    pub tag_seen_count: Option<u16>,

    pub air_protocol_tag_data: Vec<AirProtocolTagData>,

    #[tv_param = 16]
    pub access_spec_id: Option<u32>,

    pub op_spec_result: Vec<OpSpecResult>,

    pub custom: Vec<CustomParameter>,
}
#[derive(Debug, Eq, PartialEq)]
pub enum EpcDataParameter {
    EpcData(EpcData),
    Epc96([u8; 12]),
}

impl EpcDataParameter {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            EpcDataParameter::EpcData(data) => &data.epc,
            EpcDataParameter::Epc96(data) => &*data,
        }
    }
}

impl llrp_common::TlvDecodable for EpcDataParameter {
    fn decode_tlv(data: &[u8]) -> llrp_common::Result<(Self, &[u8])> {
        // First try and decoded it as a tv encoded Epc-96 parameter
        if let (Some(epc), rest) = Option::<[u8; 12]>::decode_tv(data, 13)? {
            return Ok((EpcDataParameter::Epc96(epc), rest));
        }

        // Otherwise try and decode it as a TLV encoded EPCData parameter
        let (data, rest) = EpcData::decode_tlv(data)?;
        Ok((EpcDataParameter::EpcData(data), rest))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct EpcData {
    pub epc: Vec<u8>,
}

impl llrp_common::TlvDecodable for EpcData {
    const ID: u16 = 241;

    fn decode_tlv(data: &[u8]) -> llrp_common::Result<(Self, &[u8])> {
        let (param_data, param_len) = llrp_common::parse_tlv_header(data, Self::ID)?;

        if param_data.len() < 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }

        let num_bits = u16::from_be_bytes([param_data[0], param_data[1]]) as usize;
        let num_bytes = num_bits / 8;

        if param_data.len() < 2 + num_bytes {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
        }

        let epc = EpcData {
            epc: param_data[2..][..num_bytes].into(),
        };

        Ok((epc, &data[param_len..]))
    }
}

#[llrp_parameter(id = 242)]
pub struct RfSurveyReport {
    #[tv_param = 9]
    pub ro_spec_id: Option<u16>,

    #[tv_param = 14]
    pub spec_index: Option<u16>,

    pub frequency_power_level: Vec<FrequencyPowerLevel>,

    pub custom: Vec<CustomParameter>,
}

#[llrp_parameter(id = 243)]
pub struct FrequencyPowerLevel {
    pub frequency: u32,
    pub bandwidth: u32,
    pub average_rssi: u8,
    pub peak_rssi: u8,
    pub timestamp: UTCTimestamp,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AirProtocolTagData {
    C1G2PC,
    C1G2XPCW1,
    C1G2XPCW2,
    C1G2CRC,
}
impl llrp_common::TlvDecodable for AirProtocolTagData {}

#[derive(Debug, Eq, PartialEq)]
pub enum OpSpecResult {
    C1G2OpSpecResult,
    ClientRequestOpSpecResult,
}
impl llrp_common::TlvDecodable for OpSpecResult {}

pub struct ClientRequestResponse;
impl llrp_common::TlvDecodable for ClientRequestResponse {}

pub enum AirProtocol {
    UnspecifiedAirProtocol = 0,
    EPCGlobalClass1Gen2 = 1,
    Reserved,
}
impl llrp_common::TlvDecodable for AirProtocol {}

pub enum ConfigRequestedData {
    All,
    GeneralDeviceCapabilities,
    LLRPCapabilities,
    RegulatoryCapabilities,
    AirProtocolLLRPCapabilities,
}
impl llrp_common::TlvDecodable for ConfigRequestedData {}

pub struct ReaderEventNotificationSpec;
impl llrp_common::TlvDecodable for ReaderEventNotificationSpec {}

pub struct AntennaProperties;
impl llrp_common::TlvDecodable for AntennaProperties {}

#[llrp_parameter(id = 222)]
pub struct AntennaConfiguration {
    pub antenna_id: u16,
    pub rf_receiver: Option<RfReceiver>,
    pub rf_transmitter: Option<RfTransmitter>,
    pub inventory_commands: Vec<C1G2InventoryCommand>,
    pub custom: Vec<CustomParameter>,
}

#[llrp_parameter(id = 223)]
pub struct RfReceiver {
    pub receiver_sensitivity: u16,
}

#[llrp_parameter(id = 224)]
pub struct RfTransmitter {
    pub hop_table_id: u16,
    pub channel_index: u16,
    pub transmit_power: u16,
}

#[llrp_parameter(id = 237)]
pub struct RoReportSpec {}

pub struct KeepAliveSpec;
impl llrp_common::TlvDecodable for KeepAliveSpec {}

pub struct GpoWriteData;
impl llrp_common::TlvDecodable for GpoWriteData {}

pub struct GpiPortCurrentState;
impl llrp_common::TlvDecodable for GpiPortCurrentState {}

pub struct EventsAndReports;
impl llrp_common::TlvDecodable for EventsAndReports {}

#[llrp_parameter(id = 246)]
pub struct ReaderEventNotificationData {
    pub timestamp: UTCTimestamp,
    pub connection_attempt: Option<ConnectionEventAttempt>,
}

#[llrp_parameter(id = 128)]
pub struct UTCTimestamp {
    pub microseconds: u64,
}

#[llrp_parameter(id = 256)]
pub struct ConnectionEventAttempt {
    pub status: StatusCode,
}

#[llrp_parameter(id = 330)]
pub struct C1G2InventoryCommand {
    pub tag_inventory_state_aware: u8,
    pub filter: Vec<C1G2Filter>,
    pub rf_control: Option<C1G2RfControl>,
    pub singulation_control: Option<C1G2SingulationControl>,
    pub custom: Vec<CustomParameter>,
}

#[llrp_parameter(id = 331)]
pub struct C1G2Filter {}

#[llrp_parameter(id = 335)]
pub struct C1G2RfControl {
    pub mode_index: u16,
    pub tari: u16,
}

#[llrp_parameter(id = 336)]
pub struct C1G2SingulationControl {
    // FIXME: session is stored in the first two high-bits, this should be made into an enum
    pub session: u8,
    pub tag_population: u16,
    pub tag_transit_time: u32,
    pub tag_inventory_state_aware_action: Option<TagInventoryStateAwareSingulationAction>,
}

#[llrp_parameter(id = 337)]
pub struct TagInventoryStateAwareSingulationAction {}

#[llrp_parameter(id = 338)]
#[derive(Debug, Eq, PartialEq)]
pub struct C1G2TagSpec {
    pub tag_pattern1: C1G2TargetTag,
    pub tag_pattern2: Option<C1G2TargetTag>,
}

#[llrp_parameter(id = 339)]
#[derive(Debug, Eq, PartialEq)]
pub struct C1G2TargetTag {
    pub memory_bank_and_match: u8,
    pub pointer: u16,
    pub tag_mask: BitArray,
    pub tag_data: BitArray,
}

#[llrp_parameter(id = 347)]
#[derive(Debug, Eq, PartialEq)]
pub struct C1G2BlockWrite {
    pub op_spec_id: u16,
    pub access_password: u32,
    pub memory_bank: u8,
    pub word_ptr: u16,

    #[has_length]
    pub write_data: Vec<u16>,
}
