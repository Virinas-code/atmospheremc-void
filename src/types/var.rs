//! Variable length types (Var...).
use std::{
    collections::VecDeque,
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
};

use crate::add_tuple_impl;

use super::{DataType, DataTypeDecodeError, DataTypeEncodeError, ReadBytes};

/// Segment bits of a [`VarInt`] or [`VarLong`].
const SEGMENT_BITS: i32 = 0x7F;
/// Continue bit of a [`VarInt`] or [`VarLong`].
const CONTINUE_BIT: i32 = 0x80;

/// A variable length [`i32`].
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct VarInt(pub i32);

impl Display for VarInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DataType<i32> for VarInt {
    add_tuple_impl!(VarInt i32);

    fn encode(&self, to: &mut impl Write) -> Result<(), DataTypeEncodeError> {
        #[allow(clippy::cast_sign_loss)]
        let mut value: u32 = self.0 as u32;
        // let mut bytes: Vec<u8> = Vec::new();

        loop {
            #[allow(clippy::cast_sign_loss)]
            if (value & !SEGMENT_BITS as u32) == 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                to.write_all(&[value as u8])?;
                break;
            }

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            to.write_all(&[((value & SEGMENT_BITS as u32) | CONTINUE_BIT as u32) as u8])?;

            value >>= 7;
        }

        Ok(())
    }

    fn decode(from: &mut impl Read) -> Result<Self, DataTypeDecodeError> {
        let mut value: i32 = 0;
        let mut position: u8 = 0;
        let mut current_byte: u8;

        loop {
            current_byte = from.read_byte()?;
            value |= (i32::from(current_byte) & SEGMENT_BITS) << position;

            if (i32::from(current_byte) & CONTINUE_BIT) == 0 {
                break Ok(Self(value));
            }

            position += 7;

            if position >= 32 {
                break Err(DataTypeDecodeError::VarNumberTooBig);
            }
        }
    }
}
impl TryFrom<&mut TcpStream> for VarInt {
    type Error = DataTypeDecodeError;

    fn try_from(bytes: &mut TcpStream) -> Result<Self, Self::Error> {
        let mut value: i32 = 0;
        let mut position: u8 = 0;
        let mut current_byte: u8;

        loop {
            let mut buf: [u8; 1] = [0];
            bytes.read(&mut buf).map_err(|_| {
                DataTypeDecodeError::PrematureEndOfVarNumber(VecDeque::new())
            })?;
            current_byte = buf[0];
            value |= (i32::from(current_byte) & SEGMENT_BITS) << position;

            if (i32::from(current_byte) & CONTINUE_BIT) == 0 {
                break Ok(Self(value));
            }

            position += 7;

            if position >= 32 {
                break Err(DataTypeDecodeError::VarNumberTooBig);
            }
        }
    }
}

/// A variable length [`String`].
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VarString(String);

impl DataType<String> for VarString {
    fn new(value: String) -> Self {
        Self(value)
    }

    fn get(&self) -> String {
        self.0.clone()
    }

    fn get_ref(&self) -> &String {
        &self.0
    }

    fn decode(from: &mut impl Read) -> Result<Self, DataTypeDecodeError> {
        let byte_size: usize = usize::try_from(VarInt::decode(&mut *from)?.0)?;

        Ok(Self(String::from_utf8(from.read_bytes(byte_size)?)?))
    }

    fn encode(&self, to: &mut impl Write) -> Result<(), DataTypeEncodeError> {
        VarInt::new(i32::try_from(self.0.len())?).encode(to)?;

        to.write_all(self.0.as_bytes())?;

        Ok(())
    }
}

/// A variable length [`i64`].
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct VarLong(i64);

impl DataType<i64> for VarLong {
    add_tuple_impl!(VarLong i64);

    fn decode(from: &mut impl Read) -> Result<Self, DataTypeDecodeError> {
        let mut value: i64 = 0;
        let mut position: u8 = 0;
        let mut current_byte: u8;

        loop {
            current_byte = from.read_byte()?;
            value |= (i64::from(current_byte) & i64::from(SEGMENT_BITS)) << position;

            if (i64::from(current_byte) & i64::from(CONTINUE_BIT)) == 0 {
                break Ok(Self(value));
            }

            position += 7;

            if position >= 64 {
                break Err(DataTypeDecodeError::VarNumberTooBig);
            }
        }
    }

    fn encode(&self, to: &mut impl Write) -> Result<(), DataTypeEncodeError> {
        #[allow(clippy::cast_sign_loss)]
        let mut value: u64 = self.0 as u64;

        loop {
            #[allow(clippy::cast_sign_loss)]
            if (value & !SEGMENT_BITS as u64) == 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                to.write_all(&[value as u8])?;
                break;
            }

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            to.write_all(&[((value & SEGMENT_BITS as u64) | CONTINUE_BIT as u64) as u8])?;

            value >>= 7;
        }

        Ok(())
    }
}
