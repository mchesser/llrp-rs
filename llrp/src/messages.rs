#![allow(non_camel_case_types)]

use llrp_message::llrp_message;

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

#[llrp_message(id = 47)]
pub struct SetProtocolVersion {}

#[llrp_message(id = 57)]
pub struct SetProtocolVersionResponse {}

//
// Reader device capabilities: Messages that query Reader capabilities
//

#[llrp_message(id = 1)]
pub struct GetReaderCapabilities {}

#[llrp_message(id = 11)]
pub struct GetReaderCapabilitiesResponse {}

//
// Reader operations control: Messages that control the Reader's air protocol inventory and RF
// operations
//

#[llrp_message(id = 20)]
pub struct AddRoSpec {}

#[llrp_message(id = 30)]
pub struct AddRoSpecResponse {}

#[llrp_message(id = 21)]
pub struct DeleteRoSpec {}

#[llrp_message(id = 31)]
pub struct DeleteRoSpecResponse {}

#[llrp_message(id = 22)]
pub struct StartRoSpec {}

#[llrp_message(id = 32)]
pub struct StartRoSpecResponse {}

#[llrp_message(id = 23)]
pub struct StopRoSpec {}

#[llrp_message(id = 33)]
pub struct StopRoSpecResponse {}

#[llrp_message(id = 24)]
pub struct EnableRoSpec {}

#[llrp_message(id = 34)]
pub struct EnableRoSpecResponse {}

#[llrp_message(id = 25)]
pub struct DisableRoSpec {}

#[llrp_message(id = 35)]
pub struct DisableRoSpecResponse {}

#[llrp_message(id = 26)]
pub struct GetRoSpecs {}

#[llrp_message(id = 36)]
pub struct GetRoSpecsResponse {}

//
// Access control: Messages that control the tag access perations performed by the Reader
//

#[llrp_message(id = 40)]
pub struct AddAccessSpec {}

#[llrp_message(id = 50)]
pub struct AddAccessSpecResponse {}

#[llrp_message(id = 41)]
pub struct DeleteAccessSpec {}

#[llrp_message(id = 51)]
pub struct DeleteAccessSpecResponse {}

#[llrp_message(id = 42)]
pub struct EnableAccessSpec {}

#[llrp_message(id = 52)]
pub struct EnableAccessSpecResponse {}

#[llrp_message(id = 43)]
pub struct DisableAccessSpec {}

#[llrp_message(id = 53)]
pub struct DisableAccessSpecResponse {}

#[llrp_message(id = 44)]
pub struct GetAccessSpecs {}

#[llrp_message(id = 54)]
pub struct GetAccessSpecsResponse {}

#[llrp_message(id = 45)]
pub struct ClientRequestOp {}

#[llrp_message(id = 55)]
pub struct ClientRequestOpResponse {}

//
// Reader device configuration: Messages that query/set Reader configuration, and close LLRP
// connection
//

#[llrp_message(id = 2)]
pub struct GetReaderConfig {}

#[llrp_message(id = 12)]
pub struct GetReaderConfigResponse {}

#[llrp_message(id = 3)]
pub struct SetReaderConfig {}

#[llrp_message(id = 13)]
pub struct SetReaderConfigResponse {}

#[llrp_message(id = 14)]
pub struct CloseConnection {}

#[llrp_message(id = 4)]
pub struct CloseConnectionResponse {}

//
// Reports: These are messages that carry different reports from the Reader to the Client.
// Reports include Reader device status, tag data, RF analysis report.
//

#[llrp_message(id = 60)]
pub struct GetReport {}

#[llrp_message(id = 61)]
pub struct RoAccessReport {}

#[llrp_message(id = 63)]
pub struct ReaderEventNotification {}

#[llrp_message(id = 62)]
pub struct KeepAlive {}

#[llrp_message(id = 72)]
pub struct KeepAliveAck {}

#[llrp_message(id = 64)]
pub struct EnableEventsAndReports {}

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
pub struct ErrorMessage {}
