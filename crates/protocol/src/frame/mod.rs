use crate::error::PacketError;

/// Length-prefixed protocol frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub length: u32,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(payload: Vec<u8>) -> Result<Self, PacketError> {
        let length = u32::try_from(payload.len()).map_err(|_| PacketError::TooLarge {
            actual: payload.len(),
            max: u32::MAX as usize,
        })?;

        Ok(Self { length, payload })
    }
}

/// Frame codec trait for sticky packet and split packet handling.
pub trait FrameCodec: Send + Sync {
    fn encode_frame(&self, frame: &Frame) -> Result<Vec<u8>, PacketError>;

    fn decode_frame(&self, bytes: &[u8]) -> Result<Frame, PacketError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FrameEncoder;

impl FrameEncoder {
    pub fn encode(frame: &Frame) -> Vec<u8> {
        let mut output = Vec::with_capacity(4 + frame.payload.len());
        output.extend_from_slice(&frame.length.to_be_bytes());
        output.extend_from_slice(&frame.payload);
        output
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FrameDecoder;

impl FrameDecoder {
    pub fn decode(bytes: &[u8]) -> Result<Frame, PacketError> {
        if bytes.len() < 4 {
            return Err(PacketError::InvalidPacket(
                "frame requires a 4-byte length prefix".to_owned(),
            ));
        }

        let length = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let expected = 4 + length as usize;
        if bytes.len() < expected {
            return Err(PacketError::InvalidPacket(
                "frame payload is incomplete".to_owned(),
            ));
        }

        Frame::new(bytes[4..expected].to_vec())
    }

    pub fn decode_stream(bytes: &[u8]) -> Result<(Frame, &[u8]), PacketError> {
        let frame = Self::decode(bytes)?;
        let consumed = 4 + frame.length as usize;
        Ok((frame, &bytes[consumed..]))
    }
}

impl FrameCodec for FrameEncoder {
    fn encode_frame(&self, frame: &Frame) -> Result<Vec<u8>, PacketError> {
        Ok(Self::encode(frame))
    }

    fn decode_frame(&self, bytes: &[u8]) -> Result<Frame, PacketError> {
        FrameDecoder::decode(bytes)
    }
}

pub trait FrameReader: Send + Sync {
    fn read_frame(&mut self, bytes: &[u8]) -> Result<Frame, PacketError>;
}

pub trait FrameWriter: Send + Sync {
    fn write_frame(&mut self, frame: &Frame) -> Result<Vec<u8>, PacketError>;
}
