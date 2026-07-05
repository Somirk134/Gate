use crate::{
    error::CodecError,
    message::Message,
    serializer::{JsonSerializer, Serializer},
    version::ProtocolVersion,
};

/// Message codec boundary. Codecs transform complete protocol messages only.
pub trait Codec: Send + Sync {
    fn name(&self) -> &'static str;

    fn version(&self) -> ProtocolVersion;

    fn encode(&self, message: &Message) -> Result<Vec<u8>, CodecError>;

    fn decode(&self, bytes: &[u8]) -> Result<Message, CodecError>;
}

#[derive(Debug, Clone, Default)]
pub struct JsonCodec {
    serializer: JsonSerializer,
}

impl JsonCodec {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Codec for JsonCodec {
    fn name(&self) -> &'static str {
        "json"
    }

    fn version(&self) -> ProtocolVersion {
        ProtocolVersion::V1
    }

    fn encode(&self, message: &Message) -> Result<Vec<u8>, CodecError> {
        self.serializer.serialize(message).map_err(CodecError::from)
    }

    fn decode(&self, bytes: &[u8]) -> Result<Message, CodecError> {
        self.serializer.deserialize(bytes).map_err(CodecError::from)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MessagePackCodec;

#[derive(Debug, Clone, Copy, Default)]
pub struct CborCodec;

#[derive(Debug, Clone, Copy, Default)]
pub struct ProtobufCodec;

macro_rules! reserved_codec {
    ($codec:ty, $name:literal) => {
        impl Codec for $codec {
            fn name(&self) -> &'static str {
                $name
            }

            fn version(&self) -> ProtocolVersion {
                ProtocolVersion::V1
            }

            fn encode(&self, _message: &Message) -> Result<Vec<u8>, CodecError> {
                Err(CodecError::UnsupportedCodec($name))
            }

            fn decode(&self, _bytes: &[u8]) -> Result<Message, CodecError> {
                Err(CodecError::UnsupportedCodec($name))
            }
        }
    };
}

reserved_codec!(MessagePackCodec, "messagepack");
reserved_codec!(CborCodec, "cbor");
reserved_codec!(ProtobufCodec, "protobuf");
