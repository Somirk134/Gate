use uuid::Uuid;

pub use crate::error::PacketError;

use crate::{
    codec::Codec,
    frame::{Frame, FrameDecoder, FrameEncoder},
    message::Message,
    version::ProtocolVersion,
};

/// Encoded message packet before transport I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub packet_id: Uuid,
    pub version: ProtocolVersion,
    pub payload: Vec<u8>,
    pub checksum: Option<u32>,
}

impl Packet {
    pub fn into_frame(self) -> Result<Frame, PacketError> {
        Frame::new(self.payload)
    }
}

#[derive(Debug, Clone)]
pub struct PacketBuilder {
    version: ProtocolVersion,
    checksum: Option<u32>,
}

impl PacketBuilder {
    pub fn new(version: ProtocolVersion) -> Self {
        Self {
            version,
            checksum: None,
        }
    }

    pub fn checksum(mut self, checksum: u32) -> Self {
        self.checksum = Some(checksum);
        self
    }

    pub fn build(self, payload: Vec<u8>) -> Result<Packet, PacketError> {
        PacketValidator::default().validate_payload(&payload)?;
        Ok(Packet {
            packet_id: Uuid::new_v4(),
            version: self.version,
            payload,
            checksum: self.checksum,
        })
    }

    pub fn from_message(self, codec: &dyn Codec, message: &Message) -> Result<Packet, PacketError> {
        let payload = codec
            .encode(message)
            .map_err(|error| PacketError::InvalidPacket(error.to_string()))?;
        self.build(payload)
    }
}

impl Default for PacketBuilder {
    fn default() -> Self {
        Self::new(ProtocolVersion::V1)
    }
}

#[derive(Debug, Clone)]
pub struct PacketParser {
    validator: PacketValidator,
}

impl PacketParser {
    pub fn new(validator: PacketValidator) -> Self {
        Self { validator }
    }

    pub fn parse_frame(&self, bytes: &[u8]) -> Result<Packet, PacketError> {
        let frame = FrameDecoder::decode(bytes)?;
        self.validator.validate_payload(&frame.payload)?;
        PacketBuilder::default().build(frame.payload)
    }

    pub fn parse_message(&self, codec: &dyn Codec, packet: &Packet) -> Result<Message, PacketError> {
        self.validator.validate_packet(packet)?;
        codec
            .decode(&packet.payload)
            .map_err(|error| PacketError::InvalidPacket(error.to_string()))
    }

    pub fn encode_frame(&self, packet: Packet) -> Result<Vec<u8>, PacketError> {
        let frame = packet.into_frame()?;
        Ok(FrameEncoder::encode(&frame))
    }
}

impl Default for PacketParser {
    fn default() -> Self {
        Self::new(PacketValidator::default())
    }
}

#[derive(Debug, Clone)]
pub struct PacketValidator {
    pub max_payload_len: usize,
}

impl PacketValidator {
    pub const DEFAULT_MAX_PAYLOAD_LEN: usize = 16 * 1024 * 1024;

    pub fn validate_packet(&self, packet: &Packet) -> Result<(), PacketError> {
        self.validate_payload(&packet.payload)
    }

    pub fn validate_payload(&self, payload: &[u8]) -> Result<(), PacketError> {
        if payload.is_empty() {
            return Err(PacketError::Empty);
        }

        if payload.len() > self.max_payload_len {
            return Err(PacketError::TooLarge {
                actual: payload.len(),
                max: self.max_payload_len,
            });
        }

        Ok(())
    }
}

impl Default for PacketValidator {
    fn default() -> Self {
        Self {
            max_payload_len: Self::DEFAULT_MAX_PAYLOAD_LEN,
        }
    }
}

pub trait PacketReader: Send + Sync {
    fn read_packet(&mut self, bytes: &[u8]) -> Result<Packet, PacketError>;
}

pub trait PacketWriter: Send + Sync {
    fn write_packet(&mut self, packet: Packet) -> Result<Vec<u8>, PacketError>;
}
