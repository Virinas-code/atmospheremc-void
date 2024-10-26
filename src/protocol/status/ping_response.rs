use crate::protocol::packet::ClientboundPacket;

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

    fn encode(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        let mut payload: Vec<u8> = self.time.to_be_bytes().to_vec();

        bytes.append(&mut payload);

        bytes
    }
}
