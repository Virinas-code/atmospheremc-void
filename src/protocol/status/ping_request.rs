use std::{collections::VecDeque, net::TcpStream};

use crate::{
    protocol::{
        packet::{ClientboundPacket, PacketParseError, ServerboundPacket},
        status::ping_response::CPingResponse,
    },
    state::ServerState,
    types::{DataType, DataTypeEncodeError},
};

#[derive(Debug)]
pub struct SPingRequest {
    time: i64,
}

impl ServerboundPacket for SPingRequest {
    const PACKET_ID: i32 = 1;

    fn parse(mut bytes: VecDeque<u8>) -> Result<Self, PacketParseError>
    where
        Self: Sized,
    {
        Ok(Self {
            time: i64::decode(&mut bytes)?,
        })
    }

    fn handle(
        &self,
        _server_state: ServerState,
        addr: &str,
        stream: &mut TcpStream,
    ) -> Result<ServerState, DataTypeEncodeError> {
        log::debug!(target: addr, "Received ping request at {}", self.time);

        let packet: CPingResponse = CPingResponse::new(self.time);

        packet.send(addr, stream)?;

        Ok(ServerState::Closed)
    }
}
