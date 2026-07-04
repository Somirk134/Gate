use crate::protocol::message::TunnelMessage;
use anyhow::Result;
use bytes::{BufMut, BytesMut};

pub fn encode(msg: &TunnelMessage) -> Result<BytesMut> {
    let json = serde_json::to_vec(msg)?;
    let len = json.len() as u32;
    let mut buf = BytesMut::with_capacity(4 + json.len());
    buf.put_u32(len);
    buf.extend_from_slice(&json);
    Ok(buf)
}

pub fn decode(buf: &mut BytesMut) -> Result<Option<TunnelMessage>> {
    if buf.len() < 4 {
        return Ok(None);
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if buf.len() < 4 + len {
        return Ok(None);
    }
    let _ = buf.split_to(4);
    let data = buf.split_to(len);
    let msg: TunnelMessage = serde_json::from_slice(&data)?;
    Ok(Some(msg))
}
