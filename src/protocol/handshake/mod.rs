use std::{collections::VecDeque, net::TcpStream};

use super::packet::{ServerboundPacket, StateEnum};
use crate::{
    protocol::PacketParseError,
    state::ServerState,
    types::{self, DataType, DataTypeEncodeError},
};
use handshake::SHandshake;

#[allow(clippy::module_inception)] // Handshake packet while in handshake state
mod handshake;

pub enum HandshakeServerBoundPacket {
    Handshake(SHandshake),
}

impl StateEnum for HandshakeServerBoundPacket {
    fn parse(
        packet_id: i32,
        bytes: VecDeque<u8>,
    ) -> Result<Self, super::packet::PacketParseError>
    where
        Self: Sized,
    {
        Ok(match packet_id {
            SHandshake::PACKET_ID => Self::Handshake(SHandshake::parse(bytes)?),
            other => {
                return Err(super::packet::PacketParseError::UnknownPacket(
                    other,
                    ServerState::Handshake,
                ))
            }
        })
    }

    fn handle(
        &self,
        server_state: ServerState,
        addr: &str,
        stream: &mut TcpStream,
    ) -> Result<ServerState, DataTypeEncodeError> {
        match self {
            Self::Handshake(p) => p.handle(server_state, addr, stream),
        }
    }
}

impl TryFrom<VecDeque<u8>> for HandshakeServerBoundPacket {
    type Error = PacketParseError;

    fn try_from(mut value: VecDeque<u8>) -> Result<Self, Self::Error> {
        let packet_id: i32 = types::var::VarInt::decode(&mut value)?.0;

        Self::parse(packet_id, value)
    }
}
