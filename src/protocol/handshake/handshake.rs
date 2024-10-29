use std::{collections::VecDeque, net::TcpStream};

use crate::{
    protocol::packet::{PacketParseError, ServerboundPacket},
    state::ServerState,
    types::{self, DataType, DataTypeEncodeError},
};

#[derive(Debug)]
enum State {
    Status,
    Login,
    Transfer,
}

impl TryFrom<types::var::VarInt> for State {
    type Error = types::DataTypeDecodeError;

    fn try_from(value: types::var::VarInt) -> Result<Self, Self::Error> {
        Ok(match value {
            types::var::VarInt(1) => Self::Status,
            types::var::VarInt(2) => Self::Login,
            types::var::VarInt(3) => Self::Transfer,
            other => {
                return Err(types::DataTypeDecodeError::InvalidVarIntEnumVariant {
                    variant: other,
                    enumeration: "State".to_string(),
                })
            }
        })
    }
}
#[derive(Debug)]
pub struct SHandshake {
    protocol_version: types::var::VarInt,
    server_address: types::var::VarString,
    server_port: u16,
    next_state: State,
}

impl ServerboundPacket for SHandshake {
    const PACKET_ID: i32 = 0;

    fn parse(mut bytes: VecDeque<u8>) -> Result<Self, PacketParseError>
    where
        Self: Sized,
    {
        Ok(Self {
            protocol_version: types::var::VarInt::decode(&mut bytes)?,
            server_address: types::var::VarString::decode(&mut bytes)?,
            server_port: u16::decode(&mut bytes)?,
            next_state: State::try_from(types::var::VarInt::decode(&mut bytes)?)?,
        })
    }

    fn handle(
        &self,
        _server_state: ServerState,
        addr: &str,
        _stream: &mut TcpStream,
    ) -> Result<ServerState, DataTypeEncodeError> {
        if self.protocol_version != types::var::VarInt(768) {
            log::warn!(
                target: addr,
                "Received protocol version `{0}` instead of 1.21.2 `768` ; continuing",
                self.protocol_version
            );
        }

        log::info!(
            target: addr,
            "Connected to {0:?}:{1} - Switching to {2:?} state",
            self.server_address,
            self.server_port,
            self.next_state
        );

        Ok(match self.next_state {
            State::Status => ServerState::Status,
            State::Login => ServerState::Login,
            State::Transfer => ServerState::Play,
        })
    }
}
