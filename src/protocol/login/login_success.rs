use crate::{
    protocol::packet::ServerboundPacket,
    types::{Uuid, VarString},
};

pub struct SLoginSuccess {
    uuid: Uuid,
    username: VarString,
}

impl ServerboundPacket for SLoginSuccess {
    const PACKET_ID: i32 = 0x02;

    fn parse(
        mut bytes: std::collections::VecDeque<u8>,
    ) -> Result<Self, crate::protocol::PacketParseError>
    where
        Self: Sized,
    {
        let uuid: Uuid = Uuid::try_from(&mut bytes)?;
    }

    fn handle(
        &self,
        server_state: crate::state::ServerState,
        _addr: &str,
        _stream: &mut std::net::TcpStream,
    ) -> crate::state::ServerState {
        server_state
    }
}
