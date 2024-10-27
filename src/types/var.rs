use std::{collections::VecDeque, fmt::Display, io::Read, net::TcpStream};

use crate::add_tuple_impl;

use super::{drain, DataType, DataTypeDecodeError, DataTypeEncodeError};

const SEGMENT_BITS: i32 = 0x7F;
const CONTINUE_BIT: i32 = 0x80;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct VarInt(pub i32);

impl Display for VarInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DataType<i32> for VarInt {
    add_tuple_impl!(VarInt i32);

    fn encode(&self) -> Result<Vec<u8>, DataTypeEncodeError> {
        #[allow(clippy::cast_sign_loss)]
        let mut value: u32 = self.0 as u32;
        let mut bytes: Vec<u8> = Vec::new();

        Ok(loop {
            #[allow(clippy::cast_sign_loss)]
            if (value & !SEGMENT_BITS as u32) == 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                bytes.push(value as u8);
                break bytes;
            }

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            bytes.push(((value & SEGMENT_BITS as u32) | CONTINUE_BIT as u32) as u8);

            value >>= 7;
        })
    }

    fn decode(bytes: &mut VecDeque<u8>) -> Result<Self, DataTypeDecodeError> {
        let mut value: i32 = 0;
        let mut position: u8 = 0;
        let mut current_byte: u8;

        loop {
            current_byte = bytes.pop_front().ok_or_else(|| {
                DataTypeDecodeError::PrematureEndOfVarNumber(bytes.clone())
            })?;
            value |= (i32::from(current_byte) & SEGMENT_BITS) << position;

            if (i32::from(current_byte) & CONTINUE_BIT) == 0 {
                break Ok(Self(value));
            }

            position += 7;

            if position >= 32 {
                break Err(DataTypeDecodeError::VarNumberTooBig(bytes.clone()));
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
                break Err(DataTypeDecodeError::VarNumberTooBig(VecDeque::new()));
            }
        }
    }
}

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

    fn decode(value: &mut VecDeque<u8>) -> Result<Self, DataTypeDecodeError> {
        let byte_size: usize = usize::try_from(VarInt::decode(&mut *value)?.0)?;

        Ok(Self(String::from_utf8(drain(value, 0..byte_size)?)?))
    }

    fn encode(&self) -> Result<Vec<u8>, DataTypeEncodeError> {
        let mut bytes: Vec<u8> = VarInt(i32::try_from(self.0.len())?).encode()?;

        println!("Length {bytes:?}");

        bytes.extend_from_slice(self.0.as_bytes());

        Ok(bytes)
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct VarLong(i64);

impl DataType<i64> for VarLong {
    add_tuple_impl!(VarLong i64);

    fn decode(bytes: &mut VecDeque<u8>) -> Result<Self, DataTypeDecodeError> {
        let mut value: i64 = 0;
        let mut position: u8 = 0;
        let mut current_byte: u8;

        loop {
            current_byte = bytes.pop_front().ok_or_else(|| {
                DataTypeDecodeError::PrematureEndOfVarNumber(bytes.clone())
            })?;
            value |= (i64::from(current_byte) & i64::from(SEGMENT_BITS)) << position;

            if (i64::from(current_byte) & i64::from(CONTINUE_BIT)) == 0 {
                break Ok(Self(value));
            }

            position += 7;

            if position >= 64 {
                break Err(DataTypeDecodeError::VarNumberTooBig(bytes.clone()));
            }
        }
    }

    fn encode(&self) -> Result<Vec<u8>, DataTypeEncodeError> {
        #[allow(clippy::cast_sign_loss)]
        let mut value: u64 = self.0 as u64;
        let mut bytes: Vec<u8> = Vec::new();

        Ok(loop {
            #[allow(clippy::cast_sign_loss)]
            if (value & !SEGMENT_BITS as u64) == 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                bytes.push(value as u8);
                break bytes;
            }

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            bytes.push(((value & SEGMENT_BITS as u64) | CONTINUE_BIT as u64) as u8);

            value >>= 7;
        })
    }
}
