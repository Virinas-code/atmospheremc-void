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
use core::str;
use std::{
    error::Error,
    io::{prelude::*, BufReader},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    thread::{self, JoinHandle},
};

use env_logger::Env;

type VarInt = i16;
type UnsignedShort = u16;

const SEGMENT_BITS: u8 = 0x7F;
const CONTINUE_BIT: u8 = 0x80;

fn read_varint<T: Read>(mut buf: BufReader<T>) -> Result<VarInt, Box<dyn Error>> {
    let mut value: i16 = 0;
    let mut position: i16 = 0;
    let current_byte: u8 = 0;

    loop {
        buf.read(&mut [current_byte; 1])?;
        value |= ((current_byte & SEGMENT_BITS) << position) as i16;

        if (current_byte & CONTINUE_BIT) == 0 {
            break;
        };

        position += 7;

        if position >= 32 {
            return Err(Box::from("VarInt is too big"));
        }
    }

    return Ok(value);
}

fn read_string<T: Read>(mut buf: BufReader<T>) -> Result<String, Box<dyn Error>> {
    let size: VarInt = read_varint(buf)?;

    buf.read(&mut [buffer; size]);

    str::from_utf8(v)
}

enum VarIntEnumState {
    Status,
    Login,
    Transfer,
}

impl TryFrom<VarInt> for VarIntEnumState {
    type Error = Box<dyn Error>;

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Status,
            2 => Self::Login,
            3 => Self::Transfer,
            other => return Err(Box::from(format!("Invalid state: {other}"))),
        })
    }
}

trait Serverbound {
    fn parse_impl<T: Read>(buf: BufReader<T>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

struct Handshake {
    protocol_version: VarInt,
    server_address: String,
    server_port: UnsignedShort,
    next_state: VarIntEnumState,
}

impl Serverbound for Handshake {
    fn parse_impl<T: Read>(buf: BufReader<T>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        Ok(Self {
            protocol_version: read_varint(buf)?,
            server_address: read_string(buf),
            server_port: read_unsigned_short(buf),
            next_state: VarIntEnumState::try_from(read_varint(buf)?)?,
        })
    }
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let listener: TcpListener =
        TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 25565)).unwrap();

    let mut thread_pool: Vec<JoinHandle<()>> = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => thread_pool.push(thread::spawn(|| handle_connection(stream))),
            Err(e) => log::error!("{}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    // --snip--

    let mut buf_reader = BufReader::new(&mut stream);
    let mut http_request: Vec<u8> = Vec::new();

    match buf_reader.read_to_end(&mut http_request) {
        Ok(ok) => log::debug!("Ok value: {ok}"),
        Err(e) => log::error!("ERROR: {e}"),
    }

    log::debug!("Request: {http_request:X?}");
}
