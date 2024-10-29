use std::{collections::VecDeque, net::TcpStream};

use crate::{
    protocol::{
        packet::{ClientboundPacket, PacketParseError, ServerboundPacket},
        status::status_response::CStatusResponse,
    },
    state::ServerState,
    types::DataTypeEncodeError,
};

const RESPONSE: &str = "{\"version\":{\"name\":\"1.21.2\",\"protocol\":768},\"players\":{\"max\":100,\"online\":5,\"sample\":[{\"name\":\"thinkofdeath\",\"id\":\"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"}]},\"description\":{\"text\":\"Hello, world!\"},\"favicon\":\"data:image/png;base64,<data>\",\"enforcesSecureChat\":false}";

#[derive(Debug)]
pub struct SStatusRequest {}

impl ServerboundPacket for SStatusRequest {
    const PACKET_ID: i32 = 0;

    fn parse(mut _bytes: VecDeque<u8>) -> Result<Self, PacketParseError>
    where
        Self: Sized,
    {
        Ok(Self {})
    }

    fn handle(
        &self,
        server_state: ServerState,
        addr: &str,
        stream: &mut TcpStream,
    ) -> Result<ServerState, DataTypeEncodeError> {
        log::debug!(target: addr, "Received status request");

        let packet: CStatusResponse = CStatusResponse::new(RESPONSE.to_string());

        packet.send(addr, stream)?;

        Ok(server_state)
    }
}
