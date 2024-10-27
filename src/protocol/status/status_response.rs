use crate::{protocol::packet::ClientboundPacket, types::var::VarString};

pub struct CStatusResponse {
    json_response: VarString,
}

impl CStatusResponse {
    pub fn new(response: String) -> Self {
        Self {
            json_response: VarString::from(response),
        }
    }
}

impl ClientboundPacket for CStatusResponse {
    const PACKET_ID: i32 = 0x00;

    fn encode(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        let mut payload: Vec<u8> = self.json_response.clone().into();

        bytes.append(&mut payload);

        bytes
    }
}
