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
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    thread,
    time::Duration,
};

use env_logger::fmt::style::Style;
use types::DataTypeEncodeError;

use crate::protocol::{
    HandshakeServerBoundPacket, PacketParseError, StateEnum, StatusServerBoundPacket,
};
use crate::state::ServerState;
use crate::types::{var::VarInt, DataTypeDecodeError};

mod protocol;
mod state;
mod types;

const LEGACY_PING: [u8; 25] = [
    0xFA, 0x00, 0x0B, 0x00, 0x4D, 0x00, 0x43, 0x00, 0x7C, 0x00, 0x50, 0x00, 0x69, 0x00,
    0x6E, 0x00, 0x67, 0x00, 0x48, 0x00, 0x6F, 0x00, 0x73, 0x00, 0x00,
]; // Last should be 0x74

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

fn handle_connection(mut stream: TcpStream) -> Result<(), io::Error> {
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
                } else if length == VarInt(254) {
                    let mut buf: [u8; 25] = [0; 25];
                    stream.peek(&mut buf)?;
                    if buf == LEGACY_PING {
                        log::info!(target: addr, "Handling legacy ping");

                        // Skip 25 bytes (peeked)
                        io::copy(
                            &mut Read::by_ref(&mut stream).take(25),
                            &mut io::sink(),
                        )?;

                        // Read length of rest of data
                        let mut length_buf: [u8; 2] = [0; 2];
                        stream.read_exact(&mut length_buf)?;
                        let length: u16 = u16::from_be_bytes(length_buf);

                        // We don't give a fuck
                        // Discard remaining data
                        io::copy(
                            &mut Read::by_ref(&mut stream).take(length.into()),
                            &mut io::sink(),
                        )?;

                        // Send answer
                        stream.write_all(&[0xff])?; // Kick packet
                        stream.write_all(&[0x00, 0x23])?; // Length of data after
                                                          // in characters
                        stream.write_all(&[0x00, 0xa7, 0x00, 0x31, 0x00, 0x00])?; // String
                        stream
                            .write_all(&[0x0, 0x31, 0x0, 0x32, 0x0, 0x37, 0x00, 0x00])?; // Protocol version
                        stream.write_all(&[
                            0x0, 0x31, 0x0, 0x2E, 0x0, 0x32, 0x0, 0x31, 0x0, 0x2E, 0x0,
                            0x31, 0x00, 0x00,
                        ])?; // 1.21.2
                        stream.write_all(&[
                            0x0, 0x41, 0x0, 0x74, 0x0, 0x6D, 0x0, 0x6F, 0x0, 0x73, 0x0,
                            0x70, 0x0, 0x68, 0x0, 0x65, 0x0, 0x72, 0x0, 0x65, 0x0, 0x4D,
                            0x0, 0x43, 0x0, 0x20, 0x0, 0x56, 0x0, 0x6F, 0x0, 0x69, 0x0,
                            0x64, 0x00, 0x00,
                        ])?; // AtmosphereMC - Void
                        stream.write_all(&[0x0, 0x30, 0x00, 0x00])?; // 0 players
                        stream.write_all(&[0x0, 0x30])?; // Out of 0

                        // Skip further processing: close the connection
                        break;
                    }
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
    Ok(())
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

    let result: Result<ServerState, DataTypeEncodeError> = match server_state {
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
            Ok(ServerState::Closed)
        }
        other => {
            log::error!(target: addr, "Unsupported state: {other:?}");
            Ok(other)
        }
    };

    Ok(result.unwrap_or_else(|e| {
        log::error!(target: addr, "Failed to encode packet: {e}");
        server_state
    }))
}
