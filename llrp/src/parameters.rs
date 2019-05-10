pub enum LLRPStatus {
    /// The message was recieved and processed successfully
    Success = 0,

    /// An error occured with a parameter of this message
    ParameterError = 100,

    /// An error occured with a parameter of this message
    FieldError = 101,

    /// Unexpected parameter was missing from this message
    UnexpectedParameter = 102,

    /// A required message was missing from this message
    MissingParameter = 103,

    /// Indicate that a parameter, for which there must only be one instance at the Reader, was seen
    /// more than once in this message.
    DuplicateParameter = 104,

    /// The maximum number of instances of the parameter has been exceeded at the Reader.
    OverflowParameter = 105,

    /// The maximum number of instances of the field has been exceeded at the Reader.
    OverflowField = 106,

    /// An unknown parameter was received in the message.
    UnknownParameter = 107,

    /// The field is unknown or not found at the Reader.
    UnknownField = 108,

    /// An unsupported message type was received.
    UnsupportedMessage = 109,

    /// The LLRP version in the received message is not supported by the Reader.
    UnsupportedVersion = 110,

    /// The Parameter in the received message is not supported by the Reader.
    UnsupportedParameter = 111,

    /// The message received was unexpected by the Reader.
    UnexpectedMessage = 112,

    /// An error occurred with a parameter of this parameter.
    ParameterErrorParameter = 200,

    /// An error occurred with a field of this parameter.
    FieldError = 201,

    /// An unexpected parameter was received with this message.
    UnexpectedParameter = 202,

    /// A required parameter was missing from this parameter.
    MissingParameter = 203,

    /// A parameter, for which there must only be one instance, was seen more than once in this
    /// parameter.
    DuplicateParameter = 204,

    /// The maximum number of instances of the parameter has been exceeded at the Reader.
    OverflowParameter = 205,

    /// The maximum number of instances of the field has been exceeded at the Reader.
    OverflowField = 206,

    /// An unknown parameter was received with this message.
    UnknownParameter = 207,

    /// The field is unknown or not found at the Reader.
    UnknownField = 208,

    /// An unsupported parameter was received.
    UnsupportedParameter = 209,

    /// The field value was considered invalid for a non specific reason
    InvalidField = 300,

    /// The field value did not fall within an acceptable range
    OutOfRange = 301,

    /// There is a problem on the reader
    DeviceError = 401,
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