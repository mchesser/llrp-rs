use std::io::Cursor;

use llrp_common::LLRPDecodable;

use crate::{deserializer, messages::*, parameters::*};

#[test]
fn reader_event_notifications() {
    let bytes = &[
        0x04, 0x3f, 0x00, 0x00, 0x00, 0x20, 0x3a, 0xfb, 0x30, 0xa7, 0x00, 0xf6, 0x00, 0x16, 0x00,
        0x80, 0x00, 0x0c, 0x00, 0x05, 0x88, 0x80, 0x11, 0x9f, 0x8e, 0xad, 0x01, 0x00, 0x00, 0x06,
        0x00, 0x00,
    ];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, ReaderEventNotification::ID);
    assert_eq!(raw.id, 989540519);
    assert_eq!(raw.value.len(), 32 - 10);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::ReaderEventNotification(x) => {
            let data = x.data;
            assert_eq!(data.timestamp.microseconds, 1557458516414125);

            let conn_event = data.connection_attempt.unwrap();
            assert_eq!(conn_event.status, StatusCode::M_Success);
        }
        x => panic!("Invalid message type: {}", x.id()),
    };
}

#[test]
fn enable_events_and_reports() {
    let bytes = &[0x04, 0x40, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x08];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, EnableEventsAndReports::ID);
    assert_eq!(raw.id, 8);
    assert_eq!(raw.value.len(), 0);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::EnableEventsAndReports(_) => {}
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn delete_access_spec() {
    let bytes =
        &[0x04, 0x29, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x01, 0xaf];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, DeleteAccessSpec::ID);
    assert_eq!(raw.id, 9);
    assert_eq!(raw.value.len(), 4);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::DeleteAccessSpec(x) => {
            assert_eq!(x.access_spec_id, 431);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn delete_access_spec_result_error() {
    let bytes: &[u8] = &[
        0x04, 0x33, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0x09, 0x01, 0x1f, 0x00, 0x35, 0x00,
        0x65, 0x00, 0x25, 0x4c, 0x4c, 0x52, 0x50, 0x20, 0x5b, 0x34, 0x30, 0x39, 0x5d, 0x20, 0x3a,
        0x20, 0x2f, 0x2f, 0x41, 0x63, 0x63, 0x65, 0x73, 0x73, 0x53, 0x70, 0x65, 0x63, 0x49, 0x44,
        0x20, 0x3a, 0x20, 0x69, 0x6e, 0x76, 0x61, 0x6c, 0x69, 0x64, 0x01, 0x20, 0x00, 0x08, 0x00,
        0x01, 0x01, 0x2c,
    ];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, DeleteAccessSpecResponse::ID);
    assert_eq!(raw.id, 9);
    assert_eq!(raw.value.len(), 53);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::DeleteAccessSpecResponse(x) => {
            let status = x.status;
            assert_eq!(status.status_code, StatusCode::M_FieldError);
            assert_eq!(status.error_description, "LLRP [409] : //AccessSpecID : invalid");
            let field_error = status.field_error.unwrap();
            assert_eq!(field_error.field_number, 1);
            assert_eq!(field_error.error_code, StatusCode::A_InvalidField);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn delete_ro_spec() {
    let bytes =
        &[0x04, 0x15, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x01];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, DeleteRoSpec::ID);
    assert_eq!(raw.id, 11);
    assert_eq!(raw.value.len(), 4);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::DeleteRoSpec(x) => {
            assert_eq!(x.ro_spec_id, 1);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn add_ro_spec() {
    let bytes: &[u8] = &[
        0x04, 0x14, 0x00, 0x00, 0x00, 0x5c, 0x00, 0x00, 0x00, 0x0f, 0x00, 0xb1, 0x00, 0x52, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xb2, 0x00, 0x12, 0x00, 0xb3, 0x00, 0x05, 0x01, 0x00,
        0xb6, 0x00, 0x09, 0x01, 0x00, 0x00, 0x0b, 0xb8, 0x00, 0xb7, 0x00, 0x36, 0x00, 0x01, 0x00,
        0x01, 0x00, 0xb8, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xba, 0x00, 0x25, 0x04,
        0xd2, 0x01, 0x00, 0xde, 0x00, 0x1e, 0x00, 0x01, 0x01, 0x4a, 0x00, 0x18, 0x00, 0x01, 0x4f,
        0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x01, 0x50, 0x00, 0x0b, 0x40, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x00,
    ];

    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, AddRoSpec::ID);
    assert_eq!(raw.id, 15);
    assert_eq!(raw.value.len(), 82);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::AddRoSpec(x) => {
            let spec = x.ro_spec;
            assert_eq!(spec.id, 1);
            assert_eq!(spec.priority, 0);
            assert_eq!(spec.current_state, 0);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}
