use std::io::Cursor;

use pretty_assertions::assert_eq;

use crate::{deserializer, messages::*, parameters::*, BitArray, LLRPMessage};

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
            let expected_spec = RoSpec {
                id: 1,
                priority: 0,
                current_state: 0,
                boundary_spec: RoBoundarySpec {
                    start_trigger: RoSpecStartTrigger {
                        trigger_type: 1,
                        periodic_trigger_value: None,
                        gpi_trigger_value: None,
                    },
                    stop_trigger: RoSpecStopTrigger {
                        trigger_type: 1,
                        duration_trigger_value: 3000,
                        gpi_trigger_value: None,
                    },
                },
                spec_list: vec![AiSpec {
                    antenna_ids: vec![1],
                    stop_trigger: AiSpecStopTrigger {
                        trigger_type: 0,
                        duration_trigger_value: 0,
                        gpi_trigger_value: None,
                        tag_observation_trigger_value: None,
                    },
                    inventory_specs: vec![InventorySpec {
                        spec_id: 1234,
                        protocol_id: 1,
                        antenna_configuration: vec![AntennaConfiguration {
                            antenna_id: 1,
                            rf_receiver: None,
                            rf_transmitter: None,
                            inventory_commands: vec![C1G2InventoryCommand {
                                tag_inventory_state_aware: 0,
                                filter: vec![],
                                rf_control: Some(C1G2RfControl {
                                    mode_index: 0,
                                    tari: 0,
                                }),
                                singulation_control: Some(C1G2SingulationControl {
                                    session: 1 << 6,
                                    tag_population: 1,
                                    tag_transit_time: 0,
                                    tag_inventory_state_aware_action: None,
                                }),
                                custom: vec![],
                            }],
                            custom: vec![],
                        }],
                        custom: vec![],
                    }],
                    custom: vec![],
                }],
                report_spec: None,
            };

            assert_eq!(x.ro_spec, expected_spec);
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
            assert!(x.inventory_access_report.is_empty());
            assert!(x.rf_survey_report.is_empty());
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
            assert!(x.rf_survey_report.is_empty());

            assert_eq!(x.inventory_access_report.len(), 1);
            let report_data = &x.inventory_access_report[0];
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
                air_protocol_tag_data: C1G2AirProtocolTagData::default(),
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
fn add_access_spec_read() {
    let bytes: &[u8] = &[
        0x04, 0x28, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x06, 0x8f, 0x00, 0xcf, 0x00, 0x40, 0x00,
        0x00, 0x01, 0xaf, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0xd0, 0x00, 0x07,
        0x00, 0x00, 0x01, 0x00, 0xd1, 0x00, 0x24, 0x01, 0x52, 0x00, 0x11, 0x01, 0x53, 0x00, 0x0d,
        0x60, 0x00, 0x20, 0x00, 0x08, 0xff, 0x00, 0x08, 0x0b, 0x01, 0x55, 0x00, 0x0f, 0x00, 0x6f,
        0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x10, 0x00, 0xef, 0x00, 0x05, 0x00,
    ];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, AddAccessSpec::ID);
    assert_eq!(raw.id, 1679);
    assert_eq!(raw.value.len(), 64);

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
                            tag_mask: BitArray::from_bytes(vec![0xff]),
                            tag_data: BitArray::from_bytes(vec![0x0b]),
                        },
                        tag_pattern2: None,
                    },
                    op_spec: vec![C1G2Read {
                        op_spec_id: 111,
                        access_password: 0,
                        memory_bank: 3 << 6,
                        word_ptr: 0x0000,
                        word_count: 16,
                    }
                    .into()],
                    custom: vec![],
                },
                report_spec: Some(AccessReportSpec {
                    trigger: 0,
                }),
                custom: vec![],
            };

            assert_eq!(spec, expected);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn ro_access_report_read_zero() {
    let bytes: &[u8] = &[
        0x04, 0x3d, 0x00, 0x00, 0x00, 0x32, 0x3a, 0xfb, 0x37, 0x05, 0x00, 0xf0, 0x00, 0x28, 0x8d,
        0x0b, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x02, 0x38, 0x81, 0x00, 0x01,
        0x86, 0xbc, 0x82, 0x00, 0x05, 0x88, 0x80, 0x19, 0x83, 0x92, 0xa9, 0x01, 0x5d, 0x00, 0x09,
        0x02, 0x00, 0x6f, 0x00, 0x00,
    ];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, RoAccessReport::ID);
    assert_eq!(raw.id, 989542149);
    assert_eq!(raw.value.len(), 40);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::RoAccessReport(x) => {
            assert!(x.rf_survey_report.is_empty());

            assert_eq!(x.inventory_access_report.len(), 1);
            let report_data = &x.inventory_access_report[0];
            let expected_report_data = TagReportData {
                epc_data: EpcDataParameter::Epc96([
                    0x0b, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x02, 0x38,
                ]),
                ro_spec_id: None,
                spec_index: None,
                inventory_param_spec_id: None,
                antenna_id: Some(1),
                peak_rssi: Some(188),
                channel_index: None,
                first_seen_timestamp_utc: Some(1557458648797865),
                first_seen_timestamp_uptime: None,
                last_seen_timestamp_utc: None,
                last_seen_timestamp_uptime: None,
                tag_seen_count: None,
                air_protocol_tag_data: C1G2AirProtocolTagData::default(),
                access_spec_id: None,
                op_spec_result: vec![C1G2ReadOpSpecResult {
                    result: 2,
                    op_spec_id: 111,
                    read_data: vec![],
                }
                .into()],
                custom: vec![],
            };
            assert_eq!(report_data, &expected_report_data);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn ro_access_report_read() {
    let bytes: &[u8] = &[
        0x04, 0x3d, 0x00, 0x00, 0x00, 0x52, 0x3a, 0xfb, 0x37, 0x06, 0x00, 0xf0, 0x00, 0x48, 0x8d,
        0x0b, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x02, 0x38, 0x81, 0x00, 0x01,
        0x86, 0xbc, 0x82, 0x00, 0x05, 0x88, 0x80, 0x19, 0x83, 0xab, 0x7e, 0x01, 0x5d, 0x00, 0x29,
        0x00, 0x00, 0x6f, 0x00, 0x10, 0x9d, 0x22, 0x03, 0x8a, 0x4b, 0x44, 0xa2, 0xe4, 0xd3, 0xa6,
        0x62, 0x34, 0x84, 0xae, 0x99, 0x9c, 0x21, 0x48, 0x71, 0x58, 0x6d, 0x7e, 0xc4, 0xfc, 0xc3,
        0x2a, 0x29, 0x87, 0xfa, 0x6b, 0x52, 0xab,
    ];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, RoAccessReport::ID);
    assert_eq!(raw.id, 989542150);
    assert_eq!(raw.value.len(), 72);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::RoAccessReport(x) => {
            assert!(x.rf_survey_report.is_empty());

            assert_eq!(x.inventory_access_report.len(), 1);
            let report_data = &x.inventory_access_report[0];
            let expected_report_data = TagReportData {
                epc_data: EpcDataParameter::Epc96([
                    0x0b, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x02, 0x38,
                ]),
                ro_spec_id: None,
                spec_index: None,
                inventory_param_spec_id: None,
                antenna_id: Some(1),
                peak_rssi: Some(188),
                channel_index: None,
                first_seen_timestamp_utc: Some(1557458648804222),
                first_seen_timestamp_uptime: None,
                last_seen_timestamp_utc: None,
                last_seen_timestamp_uptime: None,
                tag_seen_count: None,
                air_protocol_tag_data: C1G2AirProtocolTagData::default(),
                access_spec_id: None,
                op_spec_result: vec![C1G2ReadOpSpecResult {
                    result: 0,
                    op_spec_id: 111,
                    read_data: vec![
                        0x9d22, 0x038a, 0x4b44, 0xa2e4, 0xd3a6, 0x6234, 0x84ae, 0x999c, 0x2148,
                        0x7158, 0x6d7e, 0xc4fc, 0xc32a, 0x2987, 0xfa6b, 0x52ab,
                    ],
                }
                .into()],
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
                            tag_mask: BitArray::from_bytes(vec![0xff]),
                            tag_data: BitArray::from_bytes(vec![0x0b]),
                        },
                        tag_pattern2: None,
                    },
                    op_spec: vec![C1G2BlockWrite {
                        op_spec_id: 111,
                        access_password: 0,
                        memory_bank: 0,
                        word_ptr: 0x0003,
                        write_data: vec![0x0021],
                    }
                    .into()],
                    custom: vec![],
                },
                report_spec: Some(AccessReportSpec {
                    trigger: 0,
                }),
                custom: vec![],
            };

            assert_eq!(spec, expected);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}

#[test]
fn ro_access_report_blockwrite() {
    let bytes: &[u8] = &[
        0x04, 0x3d, 0x00, 0x00, 0x00, 0x32, 0x3a, 0xfb, 0x36, 0xed, 0x00, 0xf0, 0x00, 0x28, 0x8d,
        0x0b, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x02, 0x38, 0x81, 0x00, 0x01,
        0x86, 0xbc, 0x82, 0x00, 0x05, 0x88, 0x80, 0x19, 0x7f, 0xbd, 0xdd, 0x01, 0x62, 0x00, 0x09,
        0x00, 0x00, 0x6f, 0x00, 0x01,
    ];
    let raw = deserializer::deserialize_raw(Cursor::new(bytes)).unwrap();

    assert_eq!(raw.ver, 1);
    assert_eq!(raw.message_type, RoAccessReport::ID);
    assert_eq!(raw.id, 989542125);
    assert_eq!(raw.value.len(), 40);

    let msg = deserializer::deserialize_message(raw.message_type, &raw.value).unwrap();
    match msg {
        Message::RoAccessReport(x) => {
            assert!(x.rf_survey_report.is_empty());

            assert_eq!(x.inventory_access_report.len(), 1);
            let report_data = &x.inventory_access_report[0];
            let expected_report_data = TagReportData {
                epc_data: EpcDataParameter::Epc96([
                    0x0b, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x02, 0x38,
                ]),
                ro_spec_id: None,
                spec_index: None,
                inventory_param_spec_id: None,
                antenna_id: Some(1),
                peak_rssi: Some(188),
                channel_index: None,
                first_seen_timestamp_utc: Some(1557458648546781),
                first_seen_timestamp_uptime: None,
                last_seen_timestamp_utc: None,
                last_seen_timestamp_uptime: None,
                tag_seen_count: None,
                air_protocol_tag_data: C1G2AirProtocolTagData::default(),
                access_spec_id: None,
                op_spec_result: vec![C1G2BlockWriteOpSpecResult {
                    result: 0,
                    op_spec_id: 111,
                    words_written: 1,
                }
                .into()],
                custom: vec![],
            };
            assert_eq!(report_data, &expected_report_data);
        }
        x => panic!("Invalid message type: {}", x.id()),
    }
}
