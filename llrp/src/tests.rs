use std::io::Cursor;

use llrp_common::LLRPDecodable;

use crate::{deserializer, messages::*, parameters::*};

#[test]
fn reader_event_notifications_conn_attempt() {
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
            assert!(status.parameter_error.is_none());
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

            let start_trigger = spec.boundary_spec.start_trigger;
            assert_eq!(start_trigger.trigger_type, 1);
            assert!(start_trigger.periodic_trigger_value.is_none());
            assert!(start_trigger.gpi_trigger_value.is_none());

            let stop_trigger = spec.boundary_spec.stop_trigger;
            assert_eq!(stop_trigger.trigger_type, 1);
            assert_eq!(stop_trigger.duration_trigger_value, 3000);
            assert!(stop_trigger.gpi_trigger_value.is_none());

            assert_eq!(spec.spec_list.len(), 1);
            let ai_spec = &spec.spec_list[0];

            // FIXME: support multiple antennas
            assert_eq!(ai_spec.antenna_count, 1);
            assert_eq!(ai_spec.antenna_id, 1);

            let ai_stop_trigger = &ai_spec.stop_trigger;
            assert_eq!(ai_stop_trigger.trigger_type, 0);
            assert_eq!(ai_stop_trigger.duration_trigger_value, 0);
            assert!(ai_stop_trigger.gpi_trigger_value.is_none());
            assert!(ai_stop_trigger.tag_observation_trigger_value.is_none());

            assert_eq!(ai_spec.inventory_specs.len(), 1);
            let inventory_spec = &ai_spec.inventory_specs[0];

            assert_eq!(inventory_spec.spec_id, 1234);
            assert_eq!(inventory_spec.protocol_id, 1);

            assert_eq!(inventory_spec.antenna_configuration.len(), 1);
            let antenna_cfg = &inventory_spec.antenna_configuration[0];

            assert_eq!(antenna_cfg.antenna_id, 1);
            assert!(antenna_cfg.rf_receiver.is_none());
            assert!(antenna_cfg.rf_transmitter.is_none());

            assert_eq!(antenna_cfg.inventory_commands.len(), 1);
            let inventory_cmd = &antenna_cfg.inventory_commands[0];

            assert_eq!(inventory_cmd.tag_inventory_state_aware, 0);
            assert_eq!(inventory_cmd.filter.len(), 0);

            let rf_control = inventory_cmd.rf_control.as_ref().unwrap();
            assert_eq!(rf_control.mode_index, 0);
            assert_eq!(rf_control.tari, 0);

            let singulation_control = inventory_cmd.singulation_control.as_ref().unwrap();
            assert_eq!(singulation_control.session, 0b0100_0000);
            assert_eq!(singulation_control.tag_population, 1);
            assert_eq!(singulation_control.tag_transit_time, 0);
            assert!(singulation_control.tag_inventory_state_aware_action.is_none());
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
pub fn add_ro_spec_response() {
    let bytes = &[
        0x04, 0x1e, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x0f, 0x01, 0x1f, 0x00, 0x08, 0x00,
        0x00, 0x00, 0x00,
    ];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, AddRoSpecResponse::ID);
    assert_eq!(raw.id, 15);
    assert_eq!(raw.value.len(), 8);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::AddRoSpecResponse(x) => {
            let status = x.status;
            assert_eq!(status.status_code, StatusCode::M_Success);
            assert_eq!(status.error_description, "");
            assert!(status.field_error.is_none());
            assert!(status.parameter_error.is_none());
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn enable_ro_spec() {
    let bytes =
        &[0x04, 0x18, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x01];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, EnableRoSpec::ID);
    assert_eq!(raw.id, 17);
    assert_eq!(raw.value.len(), 4);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::EnableRoSpec(x) => {
            assert_eq!(x.ro_spec_id, 1);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn ro_access_report_simple() {
    let bytes = &[0x04, 0x3d, 0x00, 0x00, 0x00, 0x0a, 0x3a, 0xfb, 0x30, 0xa8];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, RoAccessReport::ID);
    assert_eq!(raw.id, 989540520);
    assert_eq!(raw.value.len(), 0);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::RoAccessReport(x) => {
            assert!(x.inventory_access_report_data.is_empty());
            assert!(x.rf_survey_report_data.is_empty());
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn close_connection() {
    let bytes = &[0x04, 0x0e, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x23];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, CloseConnection::ID);
    assert_eq!(raw.id, 35);
    assert_eq!(raw.value.len(), 0);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::CloseConnection(_) => {}
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
pub fn close_connection_response() {
    let bytes = &[
        0x04, 0x04, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x23, 0x01, 0x1f, 0x00, 0x08, 0x00,
        0x00, 0x00, 0x00,
    ];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, CloseConnectionResponse::ID);
    assert_eq!(raw.id, 35);
    assert_eq!(raw.value.len(), 8);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::CloseConnectionResponse(x) => {
            let status = x.status;
            assert_eq!(status.status_code, StatusCode::M_Success);
            assert_eq!(status.error_description, "");
            assert!(status.field_error.is_none());
            assert!(status.parameter_error.is_none());
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn ro_access_report_inventory() {
    let bytes: &[u8] = &[
        0x04, 0x3d, 0x00, 0x00, 0x00, 0x29, 0x3a, 0xfb, 0x30, 0xb6, 0x00, 0xf0, 0x00, 0x1f, 0x8d,
        0x0b, 0x7f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x02, 0x38, 0x81, 0x00, 0x01,
        0x86, 0xbc, 0x82, 0x00, 0x05, 0x88, 0x80, 0x19, 0x4b, 0xa9, 0xd5,
    ];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, RoAccessReport::ID);
    assert_eq!(raw.id, 989540534);
    assert_eq!(raw.value.len(), 31);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::RoAccessReport(x) => {
            assert!(x.rf_survey_report_data.is_empty());

            assert_eq!(x.inventory_access_report_data.len(), 1);
            let report_data = &x.inventory_access_report_data[0];
            let expected_report_data = TagReportData {
                epc_data: EpcDataParameter::Epc96([
                    0x0b, 0x7f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x02, 0x38,
                ]),
                ro_spec_id: None,
                spec_index: None,
                inventory_param_spec_id: None,
                antenna_id: Some(1),
                peak_rssi: Some(188),
                channel_index: None,
                first_seen_timestamp_utc: Some(1557458645133781),
                first_seen_timestamp_uptime: None,
                last_seen_timestamp_utc: None,
                last_seen_timestamp_uptime: None,
                tag_seen_count: None,
                air_protocol_tag_data: vec![],
                access_spec_id: None,
                op_spec_result: vec![],
                custom: vec![],
            };
            assert_eq!(report_data, &expected_report_data);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn add_access_spec_blockwrite() {
    let bytes: &[u8] = &[
        0x04, 0x28, 0x00, 0x00, 0x00, 0x4c, 0x00, 0x00, 0x06, 0x62, 0x00, 0xcf, 0x00, 0x42, 0x00,
        0x00, 0x01, 0xaf, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0xd0, 0x00, 0x07,
        0x00, 0x00, 0x01, 0x00, 0xd1, 0x00, 0x26, 0x01, 0x52, 0x00, 0x11, 0x01, 0x53, 0x00, 0x0d,
        0x60, 0x00, 0x20, 0x00, 0x08, 0xff, 0x00, 0x08, 0x0b, 0x01, 0x5b, 0x00, 0x11, 0x00, 0x6f,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x00, 0x21, 0x00, 0xef, 0x00, 0x05,
        0x00,
    ];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, AddAccessSpec::ID);
    assert_eq!(raw.id, 1634);
    assert_eq!(raw.value.len(), 66);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::AddAccessSpec(x) => {
            let spec = x.access_spec;
            let expected = AccessSpec {
                id: 431,
                antenna_id: 1,
                protocol_id: 1,
                current_state: false,
                ro_spec_id: 1,
                stop_trigger: AccessSpecStopTrigger {
                    trigger_type: 0,
                    operation_count: 1,
                },
                command: AccessCommand {
                    tag_spec: C1G2TagSpec {
                        tag_pattern1: C1G2TargetTag {
                            memory_bank_and_match: 0x60,
                            pointer: 0x0020,
                            tag_mask: llrp_common::BitArray {
                                bytes: vec![0xFF],
                            },
                            tag_data: llrp_common::BitArray {
                                bytes: vec![0x0b],
                            },
                        },
                        tag_pattern2: None,
                    },
                    op_spec: vec![C1G2BlockWrite {
                        op_spec_id: 111,
                        access_password: 0,
                        memory_bank: 0,
                        word_ptr: 0x0003,
                        write_data: vec![0x0021],
                    }],
                    custom: vec![],
                },
                report_spec: Some(AccessReportSpec {
                    trigger: 0,
                }),
                custom: vec![],
            };

            eprintln!("{:#?}", spec);
            eprintln!("{:#?}", expected);

            assert_eq!(spec, expected);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}
