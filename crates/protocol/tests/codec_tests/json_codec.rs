use gate_protocol::{Body, Codec, Command, JsonCodec, Message, Metadata};

#[test]
fn json_codec_preserves_message_envelope() {
    let codec = JsonCodec::new();
    let message = Message::request(
        Command::ProjectList,
        Body::Json(serde_json::json!({ "page": 1 })),
        Metadata::default(),
    );

    let encoded = codec.encode(&message).expect("json encode");
    let decoded = codec.decode(&encoded).expect("json decode");

    assert_eq!(decoded.header.message_type, message.header.message_type);
    assert_eq!(decoded.header.command, Command::ProjectList);
    assert!(matches!(decoded.body, Body::Json(_)));
}
