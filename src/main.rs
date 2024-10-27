//! # `AtmosphereMC` - Void
//!
//! The `AtmosphereMC` server.
#![warn(
    missing_docs,
    clippy::cargo_common_metadata,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::indexing_slicing
)]
use std::{
    collections::VecDeque,
    convert::identity,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use env_logger::fmt::style::Style;

use crate::protocol::{
    HandshakeServerBoundPacket, PacketParseError, StateEnum, StatusServerBoundPacket,
};
use crate::state::ServerState;
use crate::types::{var::VarInt, DataTypeDecodeError};

mod protocol;
mod state;
mod types;

fn main() {
    let mut builder = env_logger::Builder::from_default_env();

    builder
        .format(|buf, record| {
            let style = buf.default_level_style(record.level());

            let bold = Style::new().bold();
            let underline = Style::new().underline();
            let dimmed = Style::new().dimmed();

            writeln!(
                buf,
                "{bold}{underline}[{3}]{bold:#}{underline:#} {style}{bold}<{0: <5}>{bold:#}{style:#} {style}{1}{style:#} {dimmed}{2} - {4}{dimmed:#}",
                record.level(),
                record.args(),
                record.module_path().map_or("-", identity),
                record.target(),
                buf.timestamp_seconds(),
            )
        })
        .filter(None, log::LevelFilter::Trace)
        .write_style(env_logger::WriteStyle::Always)
        .format_timestamp(None)
        .format_module_path(true)
        .format_indent(Some(8))
        .init();

    log::info!(target: "Main thread", "Starting server...");

    let listener: TcpListener =
        TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 2565)).unwrap();

    log::info!(target: "Main thread", "Server ready!");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_connection(stream));
            }
            Err(e) => log::error!(target: "Main thread", "{}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));

    let addr: &str = &(match stream.peer_addr() {
        Ok(a) => format!("Client {a}"),
        Err(e) => format!("Client {e}"),
    });

    log::info!(target: addr, "Opening connection");

    let mut server_state: ServerState = ServerState::Handshake;

    loop {
        // Read packet length
        // We read it byte after byte since we can't predict its size at all
        match VarInt::try_from(&mut stream) {
            Ok(length) => {
                if length == VarInt(0) {
                    // log::warn!(target: addr, "Unknown packet of length 0!");
                    // if let Err(e) = stream.read(&mut [0; 1]) {
                    //     log::error!(target: addr, "Failed to read stream: {e}");
                    //     break;
                    // };
                    break; // Fuck it
                }
                log::trace!(target: addr, "Reading packet of length {length}");
                match handle_packet(&mut stream, length, addr, server_state) {
                    Ok(s) => {
                        if s == ServerState::Closed {
                            log::info!(target: addr, "Gracefully closing connection");
                            break;
                        };
                        server_state = s;
                    }
                    Err(e) => log::error!(target: addr, "Failed to handle packet: {e}"),
                }
            }
            Err(e) => log::error!(target: addr, "Failed to read packet length: {e}"),
        }
    }

    log::info!(target: addr, "Closing connection");
}

fn handle_packet(
    stream: &mut TcpStream,
    length: VarInt,
    addr: &str,
    server_state: ServerState,
) -> Result<ServerState, PacketParseError> {
    let mut request: Vec<u8> =
        vec![0; usize::try_from(length.0).map_err(DataTypeDecodeError::from)?];
    stream
        .read_exact(&mut request)
        .map_err(|_| DataTypeDecodeError::PrematureEnd)?;

    log::trace!(target: addr, "Request: {request:X?}");

    let deque: VecDeque<u8> = VecDeque::from(request);

    Ok(match server_state {
        ServerState::Handshake => {
            let packet = HandshakeServerBoundPacket::try_from(deque)?;
            packet.handle(server_state, addr, stream)
        }
        ServerState::Status => {
            let packet = StatusServerBoundPacket::try_from(deque)?;
            packet.handle(server_state, addr, stream)
        }
        ServerState::Closed => {
            log::error!(target: addr, "Unexpected data while in closed state");
            ServerState::Closed
        }
        other => {
            log::error!(target: addr, "Unsupported state: {other:?}");
            other
        }
    })
}
