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

pub struct CustomParameter;
impl llrp_common::LLRPDecodable for CustomParameter {
    fn decode(data: &[u8]) -> llrp_common::Result<(Self, &[u8])> {
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
impl llrp_common::LLRPDecodable for ReaderCapabilitiesRequestedData {}

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
impl llrp_common::LLRPDecodable for GeneralDeviceCapabilities {}

pub struct ReceiveSensitivityTableEntry {
    /// The index of the entry
    pub index: u16,

    /// The receive sensitivity value in dB relative to the maximum sensitivity.
    /// Possible values: 0 to 128
    pub receive_sensitivity_value: i32,
}
impl llrp_common::LLRPDecodable for ReceiveSensitivityTableEntry {}

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
impl llrp_common::LLRPDecodable for PerAntennaReceiveSensitivityRange {}

pub struct PerAntennaAirProtocolSupport {
    /// Antenna id (1-indexed). Possible values: 1 to N where N is the maximum number of antennas
    /// supported by the device
    pub antenna_id: u16,

    /// List of supported protocol IDs
    pub air_protocols_supported: Vec<AirProtocol>,
}
impl llrp_common::LLRPDecodable for PerAntennaAirProtocolSupport {}

pub struct GpioCapabilities {
    /// Number of general purpose inputs supported by the device
    pub num_gpis: u16,

    /// Number of general purpose outputs supported by the device
    pub num_gpos: u16,
}
impl llrp_common::LLRPDecodable for GpioCapabilities {}

pub struct LLRPCapabilities;
impl llrp_common::LLRPDecodable for LLRPCapabilities {}

pub struct RegulatoryCapabilities;
impl llrp_common::LLRPDecodable for RegulatoryCapabilities {}

pub struct AirProtocolLLRPCapabilities;
impl llrp_common::LLRPDecodable for AirProtocolLLRPCapabilities {}

#[llrp_parameter(id = 177)]
pub struct RoSpec {
    pub id: u32,
    pub priority: u8,
    pub current_state: u8,
    pub boundary_spec: RoBoundarySpec,
    pub list_of_specs: Vec<AiSpec>,
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
    // TODO support multiple antennas
    pub antenna_count: u16,
    pub antenna_id: u16,

    pub stop_trigger: AiSpecStopTrigger,
    pub inventory_spec: Vec<InventorySpec>,
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

pub struct AccessSpec {
    pub id: u32,
    pub antenna_id: u16,
    pub protocol_id: i32,
    pub current_state: bool,
    pub ro_spec_id: u32,
    pub stop_trigger: AccessSpecStopTrigger,
    pub access_report_spec: Option<()>,
    pub custom: Option<Vec<CustomParameter>>,
}
impl llrp_common::LLRPDecodable for AccessSpec {}

pub struct AccessSpecStopTrigger;
impl llrp_common::LLRPDecodable for AccessSpecStopTrigger {}

pub struct TagReportData;
impl llrp_common::LLRPDecodable for TagReportData {}

pub struct ClientRequestResponse;
impl llrp_common::LLRPDecodable for ClientRequestResponse {}

pub enum AirProtocol {
    UnspecifiedAirProtocol = 0,
    EPCGlobalClass1Gen2 = 1,
    Reserved,
}
impl llrp_common::LLRPDecodable for AirProtocol {}

pub enum ConfigRequestedData {
    All,
    GeneralDeviceCapabilities,
    LLRPCapabilities,
    RegulatoryCapabilities,
    AirProtocolLLRPCapabilities,
}
impl llrp_common::LLRPDecodable for ConfigRequestedData {}

pub struct ReaderEventNotificationSpec;
impl llrp_common::LLRPDecodable for ReaderEventNotificationSpec {}

pub struct AntennaProperties;
impl llrp_common::LLRPDecodable for AntennaProperties {}

pub struct AntennaConfiguration;
impl llrp_common::LLRPDecodable for AntennaConfiguration {}

#[llrp_parameter(id = 237)]
pub struct RoReportSpec {}

pub struct AccessReportSpec;
impl llrp_common::LLRPDecodable for AccessReportSpec {}

pub struct KeepAliveSpec;
impl llrp_common::LLRPDecodable for KeepAliveSpec {}

pub struct GpoWriteData;
impl llrp_common::LLRPDecodable for GpoWriteData {}

pub struct GpiPortCurrentState;
impl llrp_common::LLRPDecodable for GpiPortCurrentState {}

pub struct EventsAndReports;
impl llrp_common::LLRPDecodable for EventsAndReports {}

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
