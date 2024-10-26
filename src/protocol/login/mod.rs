use std::collections::VecDeque;

use login_success::SLoginSuccess;

use crate::{state::ServerState, types::VarInt};

use super::{packet::ServerboundPacket, PacketParseError, StateEnum};

mod login_success;

pub enum LoginServerBoundPacket {
    LoginSuccess(SLoginSuccess),
}

impl StateEnum for LoginServerBoundPacket {
    fn parse(
        packet_id: i32,
        bytes: std::collections::VecDeque<u8>,
    ) -> Result<Self, super::PacketParseError>
    where
        Self: Sized,
    {
        Ok(match packet_id {
            SLoginSuccess::PACKET_ID => Self::LoginSuccess(SLoginSuccess::parse(bytes)?),
            other => {
                return Err(PacketParseError::UnknownPacket(other, ServerState::Login))
            }
        })
    }

    fn handle(
        &self,
        server_state: crate::state::ServerState,
        addr: &str,
        stream: &mut std::net::TcpStream,
    ) -> crate::state::ServerState {
        match self {
            Self::LoginSuccess(p) => p.handle(server_state, addr, stream),
        }
    }
}

impl TryFrom<VecDeque<u8>> for LoginServerBoundPacket {
    type Error = PacketParseError;

    fn try_from(mut value: VecDeque<u8>) -> Result<Self, Self::Error> {
        let packet_id: i32 = VarInt::try_from(&mut value)?.0;

        Self::parse(packet_id, value)
    }
}
