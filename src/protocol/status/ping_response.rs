use std::io::Write;

use crate::{
    protocol::packet::ClientboundPacket,
    types::{DataType, DataTypeEncodeError},
};

pub struct CPingResponse {
    time: i64,
}

impl CPingResponse {
    pub const fn new(time: i64) -> Self {
        Self { time }
    }
}

impl ClientboundPacket for CPingResponse {
    const PACKET_ID: i32 = 0x01;

    fn encode(&self, to: &mut impl Write) -> Result<(), DataTypeEncodeError> {
        self.time.encode(to)?;

        Ok(())
    }
}
