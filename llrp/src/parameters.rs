#[allow(non_camel_case_types)]
pub enum LLRPStatus {
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

    /// Indicate that a parameter, for which there must only be one instance at the Reader, was seen
    /// more than once in this message.
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

pub struct CustomParameter;

pub enum ReaderCapabilitiesRequestedData {
    All,
    GeneralDeviceCapabilities,
    LLRPCapabilities,
    RegulatoryCapabilities,
    AirProtocolLLRPCapabilities,
}

pub struct GeneralDeviceCapabilities;
pub struct LLRPCapabilities;
pub struct RegulatoryCapabilities;
pub struct AirProtocolLLRPCapabilities;

pub struct RoSpec;

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

pub struct AccessSpecStopTrigger;

pub struct TagReportData;

pub struct ClientRequestResponse;

pub enum AirProtocol {
    UnspecifiedAirProtocol = 0,
    EPCGlobalClass1Gen2 = 1,
    Reserved,
}

pub enum ConfigRequestedData {
    All,
    GeneralDeviceCapabilities,
    LLRPCapabilities,
    RegulatoryCapabilities,
    AirProtocolLLRPCapabilities,
}

pub struct ReaderEventNotificationSpec;
pub struct AntennaProperties;
pub struct AntennaConfiguration;
pub struct RoReportSpec;
pub struct AccessReportSpec;
pub struct KeepAliveSpec;
pub struct GpoWriteData;
pub struct GpiPortCurrentState;
pub struct EventsAndReports;

pub struct ReaderEventNotificationData;