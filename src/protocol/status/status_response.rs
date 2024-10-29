use std::io::Write;

use crate::{
    protocol::packet::ClientboundPacket,
    types::{var::VarString, DataType, DataTypeEncodeError},
};

pub struct CStatusResponse {
    json_response: VarString,
}

impl CStatusResponse {
    pub fn new(response: String) -> Self {
        Self {
            json_response: VarString::new(response),
        }
    }
}

impl ClientboundPacket for CStatusResponse {
    const PACKET_ID: i32 = 0x00;

    fn encode(&self, to: &mut impl Write) -> Result<(), DataTypeEncodeError> {
        self.json_response.clone().encode(to)?;
        Ok(())
    }
}
