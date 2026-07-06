use anyhow::{anyhow, Context, Result};
use gate_protocol::{
    Body, Command, Frame, FrameEncoder, Message, MessageType, Metadata, ProtocolBuilder,
    ProtocolManager,
};
use serde_json::{json, Value};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

/// Creates an Alpha V1 protocol manager with JSON codec.
pub fn alpha_protocol() -> ProtocolManager {
    ProtocolBuilder::new().build()
}

/// Builds a request message.
pub fn request(command: Command, body: Value) -> Message {
    Message::request(command, Body::Json(body), Metadata::default())
}

/// Builds a response that keeps the original request id.
pub fn response_for(request: &Message, body: Value) -> Message {
    let mut response = Message::new(
        MessageType::Response,
        request.header.command.clone(),
        Body::Json(body),
        Metadata::default(),
    );
    response.header.request_id = request.header.request_id;
    response
}

/// Builds a protocol event.
pub fn event(name: &str, payload: Value) -> Message {
    Message::new(
        MessageType::Event,
        Command::Custom(name.to_string()),
        Body::Json(payload),
        Metadata::default(),
    )
}

/// Extracts a JSON body from a protocol message.
pub fn json_body(message: &Message) -> Result<Value> {
    match &message.body {
        Body::Json(value) => Ok(value.clone()),
        Body::Empty => Ok(Value::Null),
        _ => Err(anyhow!("expected JSON protocol body")),
    }
}

/// Writes a single framed protocol message to a TCP stream.
pub async fn write_message(
    stream: &mut TcpStream,
    protocol: &ProtocolManager,
    message: &Message,
) -> Result<()> {
    let payload = protocol.encode(message)?;
    let frame = Frame::new(payload)?;
    let bytes = FrameEncoder::encode(&frame);
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}

/// Reads a single framed protocol message from a TCP stream.
pub async fn read_message(
    stream: &mut TcpStream,
    protocol: &ProtocolManager,
) -> Result<Option<Message>> {
    let mut length = [0_u8; 4];
    match stream.read_exact(&mut length).await {
        Ok(_) => {}
        Err(source) if source.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(source) => return Err(source).context("read protocol frame length"),
    }

    let length = u32::from_be_bytes(length) as usize;
    let mut payload = vec![0_u8; length];
    stream
        .read_exact(&mut payload)
        .await
        .context("read protocol frame payload")?;
    Ok(Some(protocol.decode(&payload)?))
}

/// Builds a successful response payload.
pub fn ok(payload: Value) -> Value {
    json!({
        "ok": true,
        "data": payload
    })
}

/// Builds a failed response payload.
pub fn err(code: &str, message: &str) -> Value {
    json!({
        "ok": false,
        "error": {
            "code": code,
            "message": message
        }
    })
}
