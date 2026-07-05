use gate_protocol::{
    Body, Command, Message, Metadata, ProtocolBuilder, ProtocolVersion, VersionNegotiation,
};

#[test]
fn protocol_manager_round_trips_v1_message() {
    let manager = ProtocolBuilder::new().build();
    let message = Message::request(
        Command::SystemHealth,
        Body::Json(serde_json::json!({ "status": "probe" })),
        Metadata::default(),
    );

    let encoded = manager.encode(&message).expect("message should encode");
    let decoded = manager.decode(&encoded).expect("message should decode");

    assert_eq!(decoded.header.command, Command::SystemHealth);
    assert_eq!(decoded.header.protocol_version, ProtocolVersion::V1);
}

#[test]
fn version_negotiation_selects_v1() {
    let mut manager = ProtocolBuilder::new().build();
    let negotiation = VersionNegotiation::v1();

    let selected = manager
        .negotiate(&negotiation, &[ProtocolVersion::V1])
        .expect("v1 should negotiate");

    assert_eq!(selected, ProtocolVersion::V1);
    assert_eq!(manager.active_version(), ProtocolVersion::V1);
}
