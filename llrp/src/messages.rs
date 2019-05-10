#![allow(non_camel_case_types)]

use llrp_message::llrp_message;

use crate::parameters::*;

//
// Protocol version management
//

/// Queries the LLRP version supported by the Reader
#[llrp_message(id = 46)]
pub struct GetSupportedVersion;

/// Response to the `GET_SUPPORTED_VERSION` message
#[llrp_message(id = 56)]
pub struct GetSupportedVersionResponse {
    /// The currently negotiated protocol version
    pub current_version: u8,

    /// The maximum supported protocol version
    pub supported_version: u8,
}

/// Sets the protocol version for the current connection
#[llrp_message(id = 47)]
pub struct SetProtocolVersion {
    /// The desired protocol version
    pub protocol_version: u8,
}

/// Response to the `SET_PROTOCOL_VERSION` message
#[llrp_message(id = 57)]
pub struct SetProtocolVersionResponse {
    /// Reader status
    pub status: LLRPStatus,
}

//
// Reader device capabilities: Messages that query Reader capabilities
//

/// Request capabilities from the reader
#[llrp_message(id = 1)]
pub struct GetReaderCapabilities {
    /// Configures whether to query all or only a subset of all the capabilties from the reader
    pub requested_data: ReaderCapabilitiesRequestedData,

    /// Optional custom parameters
    pub custom: Option<Vec<CustomParameter>>,
}

/// Response to the `GET_READER_CAPABILITIES` message
#[llrp_message(id = 11)]
pub struct GetReaderCapabilitiesResponse {
    /// Reader status
    pub status: Option<LLRPStatus>,

    /// Reader general device capabilities
    pub general: Option<GeneralDeviceCapabilities>,

    /// Reader LLRP capabilities
    pub llrp: Option<LLRPCapabilities>,

    /// Reader regulatory capabilities
    pub regulatory: Option<RegulatoryCapabilities>,

    /// Reader Air protocol LLRP capabilities
    pub air_protocol: Option<AirProtocolLLRPCapabilities>,

    /// Optional custom parameters
    pub custom: Option<Vec<CustomParameter>>,
}

//
// Reader operations control: Messages that control the Reader's air protocol inventory and RF
// operations
//

/// Communicates the information of a ROSpec to the Reader.
#[llrp_message(id = 20)]
pub struct AddRoSpec {
    /// The ROSpec to add
    pub ro_spec: RoSpec,
}

/// Response to the `ADD_RO_SPEC` message
#[llrp_message(id = 30)]
pub struct AddRoSpecResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Deletes an ROSpec
#[llrp_message(id = 21)]
pub struct DeleteRoSpec {
    /// The identifier of the ROSpec to delete (0 indicates to delete all ROSpecs)
    pub ro_spec_id: u32,
}

/// Response to the `DELETE_RO_SPEC` message
#[llrp_message(id = 31)]
pub struct DeleteRoSpecResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Starts an ROSpec if it is in the enabled state.
#[llrp_message(id = 22)]
pub struct StartRoSpec {
    /// The identifier of the ROSpec to start (0 is disallowed)
    pub ro_spec_id: u32,
}

/// Response to the `START_RO_SPEC` message
#[llrp_message(id = 32)]
pub struct StartRoSpecResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Sets the ROSpec specified by `ro_spec_id` to inactive.
#[llrp_message(id = 23)]
pub struct StopRoSpec {
    /// The identifier of the ROSpec to stop (0 is disallowed)
    pub ro_spec_id: u32,
}

/// Response to the `STOP_RO_SPEC` message
#[llrp_message(id = 33)]
pub struct StopRoSpecResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Enables a ROSpec
#[llrp_message(id = 24)]
pub struct EnableRoSpec {
    /// The identifier of the ROSpec to enable. (If set to 0, all ROSpecs are enabled).
    pub ro_spec_id: u32,
}

/// Response to the `ENABLE_RO_SPEC` message
#[llrp_message(id = 34)]
pub struct EnableRoSpecResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Disables a ROSpec
#[llrp_message(id = 25)]
pub struct DisableRoSpec {
    /// The identifier of the ROSpec to disable. (If set to 0, all ROSpecs are disabled).
    pub ro_spec_id: u32,
}

/// Response to the `DISABLE_RO_SPEC` message
#[llrp_message(id = 35)]
pub struct DisableRoSpecResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Retrieve all ROSpecs that have been configured on the reader.
#[llrp_message(id = 26)]
pub struct GetRoSpecs;

/// Response to the `GET_RO_SPECS` message
#[llrp_message(id = 36)]
pub struct GetRoSpecsResponse {
    /// Reader status
    pub status: LLRPStatus,

    /// List of ROSpecs in the order in which they were added
    pub ro_specs: Vec<RoSpec>,
}

//
// Access control: Messages that control the tag access perations performed by the Reader
//

/// Creates a new AccessSpec at the Reader
#[llrp_message(id = 40)]
pub struct AddAccessSpec {
    /// The access spec to add
    pub access_spec: AccessSpec,
}

/// Response to the `ADD_ACCESS_SPEC` message
#[llrp_message(id = 50)]
pub struct AddAccessSpecResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Deletes an AccessSpec on the reader
#[llrp_message(id = 41)]
pub struct DeleteAccessSpec {
    /// The identifier of the AccessSpec to delete. (If set to 0, all AccessSpecs are deleted).
    pub access_spec_id: u32,
}

/// Response to the `DELETE_ACCESS_SPEC` message
#[llrp_message(id = 51)]
pub struct DeleteAccessSpecResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Enables an AccessSpec
#[llrp_message(id = 42)]
pub struct EnableAccessSpec {
    /// The identifier of the AccessSpec to enabled. (If set to 0, all AccessSpecs are enabled).
    pub access_spec_id: u32,
}

/// Response to the `ENABLE_ACCESS_SPEC` message
#[llrp_message(id = 52)]
pub struct EnableAccessSpecResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Disables an AccessSpec
#[llrp_message(id = 43)]
pub struct DisableAccessSpec {
    /// The identifier of the AccessSpec to disable. (If set to 0, all AccessSpecs are disabled).
    pub access_spec_id: u32,
}

/// Response to the `DISABLE_ACCESS_SPEC` message
#[llrp_message(id = 53)]
pub struct DisableAccessSpecResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Retrieve all AccessSpecs that have been configured on the reader.
#[llrp_message(id = 44)]
pub struct GetAccessSpecs;

/// Response to the `GET_ACCESS_SPECS` message
#[llrp_message(id = 54)]
pub struct GetAccessSpecsResponse {
    /// Reader status
    pub status: LLRPStatus,

    /// List of AccessSpecs in the order in which they were created at the reader
    pub access_specs: Vec<AccessSpec>,
}

/// Message sent by the Reader to the Client upon executing a ClientRequestOpSpec OpSpec.
#[llrp_message(id = 45)]
pub struct ClientRequestOp {
    /// Contains singulation results and the results of the OpSpecs executed till that point.
    pub tag_report: Vec<TagReportData>,
}

/// Response to the `CLIENT_REQUEST_OP` message. Sent from the Client to the Reader
#[llrp_message(id = 55)]
pub struct ClientRequestOpResponse {
    /// Client response
    pub response: ClientRequestResponse,
}

//
// Reader device configuration: Messages that query/set Reader configuration, and close LLRP
// connection
//

/// Get current configuration information of the Reader
#[llrp_message(id = 2)]
pub struct GetReaderConfig {
    /// Data to request
    pub requested_data: ConfigRequestedData,

    /// Specifies which antenna to get information for (or all if = 0)
    /// Ignored when `requested_data` != `0` or `2` or `3`
    pub antenna_id: u16,

    /// Specifies which GPI port to get information for (or all if = 0)
    /// Ignored when `requested_data` != 0` or `9`
    pub gpi_port_num: u16,

    /// Specifies which GPO port to get information for (or all if = 0)
    /// Ignored when `requested_data` != 0` or `10`
    pub gpo_port_num: u16,

    /// Optional custom parameters
    pub custom: Option<Vec<CustomParameter>>,
}

/// Response to the `GET_READER_CONFIG` message
#[llrp_message(id = 12)]
pub struct GetReaderConfigResponse {
    /// Reader status
    pub status: LLRPStatus,

    pub reader_event_notification_spec: Option<ReaderEventNotificationSpec>,
    pub antenna_properties: Option<AntennaProperties>,
    pub antenna_configuration: Option<AntennaConfiguration>,
    pub ro_report_spec: Option<RoReportSpec>,
    pub access_report_spec: Option<AccessReportSpec>,
    pub keep_alive_spec: Option<KeepAliveSpec>,
    pub gpo_write_data: Option<GpoWriteData>,
    pub gpi_port_current_state: Option<GpiPortCurrentState>,
    pub events_and_reports: Option<EventsAndReports>,

    /// Optional custom parameters
    pub custom: Option<Vec<CustomParameter>>,
}

/// Sets the reader configuration using the parameters specified in the command
#[llrp_message(id = 3)]
pub struct SetReaderConfig {
    /// If true the reader will set all configurable values to factory defaults before applying the
    /// remaining parameters
    pub reset_to_factory_default: bool,

    pub reader_event_notification_spec: Option<ReaderEventNotificationSpec>,
    pub antenna_properties: Option<AntennaProperties>,
    pub antenna_configuration: Option<AntennaConfiguration>,
    pub ro_report_spec: Option<RoReportSpec>,
    pub access_report_spec: Option<AccessReportSpec>,
    pub keep_alive_spec: Option<KeepAliveSpec>,
    pub gpo_write_data: Option<GpoWriteData>,
    pub gpi_port_current_state: Option<GpiPortCurrentState>,
    pub events_and_reports: Option<EventsAndReports>,

    /// Optional custom parameters
    pub custom: Option<Vec<CustomParameter>>,
}

/// Response to the `SET_READER_CONFIG` message
#[llrp_message(id = 13)]
pub struct SetReaderConfigResponse {
    /// Reader status
    pub status: LLRPStatus,
}

/// Instruct the reader to gracefully close its connection with the client
#[llrp_message(id = 14)]
pub struct CloseConnection;

/// Response to the `CLOSE_CONNECTION` message
#[llrp_message(id = 4)]
pub struct CloseConnectionResponse {
    /// Reader status
    pub status: LLRPStatus,
}

//
// Reports: These are messages that carry different reports from the Reader to the Client.
// Reports include Reader device status, tag data, RF analysis report.
//

/// Request tag reports from the Reader.
#[llrp_message(id = 60)]
pub struct GetReport;

/// Contains the results of the RO and Access operations
#[llrp_message(id = 61)]
pub struct RoAccessReport {
    pub inventory_access_report_data: Option<Vec<TagReportData>>,

    pub rf_survey_report_data: Option<Vec<()>>,

    /// Optional custom parameters
    pub custom: Option<Vec<CustomParameter>>,
}

/// Message issued by the reader to the client
#[llrp_message(id = 62)]
pub struct KeepAlive;

/// Generated by the client in response to the `KEEPALIVE` messages sent by the reader
#[llrp_message(id = 72)]
pub struct KeepAliveAck;

/// Generated by the reader and sent to the client whenever an event that the client is subscribed
/// to occurs.
#[llrp_message(id = 63)]
pub struct ReaderEventNotification {
    /// The data associated with the event
    pub data: ReaderEventNotificationData,
}

/// Used to inform the Reader that it can remove its hold on event and report messages.
#[llrp_message(id = 64)]
pub struct EnableEventsAndReports;

//
// Custom Extension: This is a common mechanism for messages that contain vendor defined content.
//

#[llrp_message(id = 1023)]
pub struct CustomMessage {}

// Errors: Typically the errors in the LLRP defined messages are conveyed inside of the
// responses from the Reader. However, in cases where the message received by the Reader contains an
// unsupported message type, or a CUSTOM_MESSAGE with unsupported parameters or fields, the
// Reader SHALL respond with this generic error message.

#[llrp_message(id = 1000)]
pub struct ErrorMessage {
    error: LLRPStatus
}
