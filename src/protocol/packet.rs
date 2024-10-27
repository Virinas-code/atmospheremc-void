use std::collections::VecDeque;
use std::io::Write;
use std::net::TcpStream;

use thiserror::Error;

use crate::state::ServerState;
use crate::types::{self, var::VarInt};

#[derive(Error, Debug)]
pub enum PacketParseError {
    #[error(transparent)]
    DataTypeDecodeError(#[from] types::DataTypeDecodeError),

    #[error("Unknown packet: {0:X?} in {1:?}")]
    UnknownPacket(i32, ServerState),
}

pub trait ServerboundPacket {
    const PACKET_ID: i32;

    fn parse(bytes: VecDeque<u8>) -> Result<Self, PacketParseError>
    where
        Self: Sized;

    fn handle(
        &self,
        server_state: ServerState,
        addr: &str,
        stream: &mut TcpStream,
    ) -> ServerState;
}

pub trait StateEnum: TryFrom<VecDeque<u8>> {
    fn parse(packet_id: i32, bytes: VecDeque<u8>) -> Result<Self, PacketParseError>
    where
        Self: Sized;

    fn handle(
        &self,
        server_state: ServerState,
        addr: &str,
        stream: &mut TcpStream,
    ) -> ServerState;
}

pub trait ClientboundPacket {
    const PACKET_ID: i32;

    fn encode(&self) -> Vec<u8>;

    fn send(&self, addr: &str, stream: &mut TcpStream) {
        let mut bytes: Vec<u8> = VarInt(Self::PACKET_ID).into();

        bytes.append(&mut self.encode());

        let length: VarInt = VarInt(match bytes.len().try_into() {
            Ok(v) => v,
            Err(e) => {
                log::error!(target: addr, "Failed to encode packet {0} ({1:X?}) length {2}: {e}", Self::PACKET_ID, bytes, bytes.len());
                return;
            }
        });

        let mut payload: Vec<u8> = length.into();
        payload.append(&mut bytes);

        log::trace!(target: addr, "Sending packet {0} ({1:X?})", Self::PACKET_ID, payload);

        if let Err(e) = stream.write_all(&payload) {
            log::warn!(target: addr, "Failed to send packet {0} ({1:X?}): {e}", Self::PACKET_ID, payload);
        }

        log::trace!(target: addr, "Packet {0} sent", Self::PACKET_ID);
    }
}
