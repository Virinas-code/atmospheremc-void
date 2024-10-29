use std::{collections::VecDeque, net::TcpStream};

use ping_request::SPingRequest;
use status_request::SStatusRequest;

use super::packet::{ServerboundPacket, StateEnum};
use crate::{
    protocol::PacketParseError,
    state::ServerState,
    types::{self, DataType, DataTypeEncodeError},
};

mod ping_request;
mod ping_response;
mod status_request;
mod status_response;

pub enum StatusServerBoundPacket {
    StatusRequest(SStatusRequest),
    PingRequest(SPingRequest),
}

impl StateEnum for StatusServerBoundPacket {
    fn parse(
        packet_id: i32,
        bytes: VecDeque<u8>,
    ) -> Result<Self, super::packet::PacketParseError>
    where
        Self: Sized,
    {
        Ok(match packet_id {
            SStatusRequest::PACKET_ID => {
                Self::StatusRequest(SStatusRequest::parse(bytes)?)
            }
            SPingRequest::PACKET_ID => Self::PingRequest(SPingRequest::parse(bytes)?),
            other => {
                return Err(super::packet::PacketParseError::UnknownPacket(
                    other,
                    ServerState::Status,
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
            Self::StatusRequest(p) => p.handle(server_state, addr, stream),
            Self::PingRequest(p) => p.handle(server_state, addr, stream),
        }
    }
}

impl TryFrom<VecDeque<u8>> for StatusServerBoundPacket {
    type Error = PacketParseError;

    fn try_from(mut value: VecDeque<u8>) -> Result<Self, Self::Error> {
        let packet_id: i32 = types::var::VarInt::decode(&mut value)?.0;

        Self::parse(packet_id, value)
    }
}
