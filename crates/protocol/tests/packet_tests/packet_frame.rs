use gate_protocol::{
    Body, Command, JsonCodec, Message, Metadata, PacketBuilder, PacketParser, ProtocolVersion,
};

#[test]
fn packet_parser_handles_length_prefixed_frame() {
    let codec = JsonCodec::new();
    let message = Message::request(
        Command::SystemVersion,
        Body::Json(serde_json::json!({ "client": "test" })),
        Metadata::default(),
    );

    let packet = PacketBuilder::new(ProtocolVersion::V1)
        .from_message(&codec, &message)
        .expect("packet build");
    let parser = PacketParser::default();
    let frame = parser.encode_frame(packet).expect("frame encode");
    let parsed = parser.parse_frame(&frame).expect("frame parse");
    let decoded = parser
        .parse_message(&codec, &parsed)
        .expect("message parse");

    assert_eq!(decoded.header.command, Command::SystemVersion);
}
