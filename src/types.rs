use std::{
    collections::VecDeque, fmt::Display, io::Read, net::TcpStream, num::TryFromIntError,
    ops::RangeBounds, string::FromUtf8Error,
};

use thiserror::Error;

fn drain<R>(bytes: &mut VecDeque<byte>, range: R) -> Result<Vec<u8>, DataTypeDecodeError>
where
    R: RangeBounds<usize> + Iterator,
{
    let mut output: Vec<u8> = Vec::new();
    for _ in range {
        output.push(bytes.pop_front().ok_or_else(|| {
            DataTypeDecodeError::PrematureEndOfVarNumber(bytes.clone())
        })?);
    }
    Ok(output)
}

#[allow(non_camel_case_types)]
pub type byte = u8;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DataTypeDecodeError {
    #[error("VarNumber ended prematurely: {0:X?}")]
    PrematureEndOfVarNumber(VecDeque<byte>),

    #[error("VarNumber too big: {0:X?}")]
    VarNumberTooBig(VecDeque<byte>),

    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),

    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Premature end of stream while reading a fixed length data type")]
    PrematureEnd,

    #[error("Invalid VarInt Enum variant: {variant} for {enumeration}")]
    InvalidVarIntEnumVariant {
        variant: VarInt,
        enumeration: String,
    },
}

#[allow(dead_code)] // I swear this is useful
pub trait DataType<'a, Inner>:
    TryFrom<&'a mut VecDeque<u8>, Error = DataTypeDecodeError>
    + From<Inner>
    + Into<Vec<u8>>
    + Clone
{
}

const SEGMENT_BITS: i32 = 0x7F;
const CONTINUE_BIT: i32 = 0x80;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct VarInt(pub i32);

impl Display for VarInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DataType<'_, i32> for VarInt {}
impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl From<VarInt> for Vec<u8> {
    fn from(val: VarInt) -> Self {
        #[allow(clippy::cast_sign_loss)]
        let mut value: u32 = val.0 as u32;
        let mut bytes: Self = Self::new();

        loop {
            #[allow(clippy::cast_sign_loss)]
            if (value & !SEGMENT_BITS as u32) == 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                bytes.push(value as u8);
                break bytes;
            }

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            bytes.push(((value & SEGMENT_BITS as u32) | CONTINUE_BIT as u32) as u8);

            value >>= 7;
        }
    }
}
impl TryFrom<&mut VecDeque<byte>> for VarInt {
    type Error = DataTypeDecodeError;

    fn try_from(bytes: &mut VecDeque<byte>) -> Result<Self, Self::Error> {
        let mut value: i32 = 0;
        let mut position: u8 = 0;
        let mut current_byte: byte;

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
        let mut current_byte: byte;

        loop {
            let mut buf: [byte; 1] = [0];
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

#[derive(PartialEq, Debug, Clone, Copy)]
struct VarLong(i64);

impl DataType<'_, i64> for VarLong {}
impl From<i64> for VarLong {
    fn from(value: i64) -> Self {
        Self(value)
    }
}
impl From<VarLong> for Vec<u8> {
    fn from(val: VarLong) -> Self {
        #[allow(clippy::cast_sign_loss)]
        let mut value: u64 = val.0 as u64;
        let mut bytes: Self = Self::new();

        loop {
            #[allow(clippy::cast_sign_loss)]
            if (value & !SEGMENT_BITS as u64) == 0 {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                bytes.push(value as u8);
                break bytes;
            }

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            bytes.push(((value & SEGMENT_BITS as u64) | CONTINUE_BIT as u64) as u8);

            value >>= 7;
        }
    }
}
impl TryFrom<&mut VecDeque<byte>> for VarLong {
    type Error = DataTypeDecodeError;

    fn try_from(bytes: &mut VecDeque<byte>) -> Result<Self, Self::Error> {
        let mut value: i64 = 0;
        let mut position: u8 = 0;
        let mut current_byte: byte;

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
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VarString(String);

impl Display for VarString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DataType<'_, String> for VarString {}
impl From<String> for VarString {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<VarString> for Vec<u8> {
    fn from(val: VarString) -> Self {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        // TODO: Don't
        let mut bytes: Self = VarInt(val.0.len() as i32).into();

        println!("Length {bytes:?}");

        bytes.extend_from_slice(val.0.as_bytes());

        bytes
    }
}
impl TryFrom<&mut VecDeque<byte>> for VarString {
    type Error = DataTypeDecodeError;

    fn try_from(bytes: &mut VecDeque<byte>) -> Result<Self, Self::Error> {
        let byte_size: usize = usize::try_from(VarInt::try_from(&mut *bytes)?.0)?;

        Ok(Self(String::from_utf8(drain(bytes, 0..byte_size)?)?))
    }
}

pub fn parse_unsigned_short(
    bytes: &mut VecDeque<byte>,
) -> Result<u16, DataTypeDecodeError> {
    Ok(
        (u16::from(bytes.pop_front().ok_or(DataTypeDecodeError::PrematureEnd)?) << 8)
            | u16::from(bytes.pop_front().ok_or(DataTypeDecodeError::PrematureEnd)?),
    )
}

pub fn parse_long(bytes: &mut VecDeque<byte>) -> Result<i64, DataTypeDecodeError> {
    Ok(i64::from_be_bytes(
        drain(bytes, 0..8)?
            .try_into()
            .map_err(|_| DataTypeDecodeError::PrematureEnd)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_fixed_length_parse() {
    //     fn old_parse_unsigned_short(
    //         bytes: &mut VecDeque<byte>,
    //     ) -> Result<u16, DataTypeDecodeError> {
    //         Ok(
    //             (u16::from(bytes.pop_front().ok_or(DataTypeDecodeError::PrematureEnd)?)
    //                 << 8)
    //                 | u16::from(
    //                     bytes.pop_front().ok_or(DataTypeDecodeError::PrematureEnd)?,
    //                 ),
    //         )
    //     }
    //     let tests: [(Vec<u8>, u16); 8] = [
    //         (vec![0x00, 0x00], 0),
    //         (vec![0x00, 0x01], 1),
    //         (vec![0x00, 0x02], 2),
    //         (vec![0x00, 0x2A], 42),
    //         (vec![0x00, 0xff], 255),
    //         (vec![0x01, 0xff], 256),
    //         (vec![0x10, 0x92], 4242),
    //         (vec![0xff, 0xff], 65535),
    //     ];
    //     for (bytes, value) in tests {
    //         let old: Result<u16, DataTypeDecodeError> =
    //             old_parse_unsigned_short(&mut VecDeque::from(bytes.clone()));
    //         assert_eq!(
    //             old,
    //             Ok(value),
    //             "Old parse returned {old:?} instead of {value} for {bytes:?}"
    //         );

    //         let new: Result<u16, DataTypeDecodeError> =
    //             parse_unsigned_short(&mut VecDeque::from(bytes.clone()));
    //         assert_eq!(
    //             new,
    //             Ok(value),
    //             "New parse returned {new:?} instead of {value} for {bytes:?}"
    //         );
    //     }
    // }

    #[test]
    fn test_fixed_long() {
        let tests: [(Vec<u8>, i64); 1] = [(
            vec![0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            9_223_372_036_854_775_807,
        )];
        for (bytes, value) in tests {
            let parsed: Result<i64, DataTypeDecodeError> =
                parse_long(&mut VecDeque::from(bytes.clone()));
            assert_eq!(
                parsed,
                Ok(value),
                "Parse returned {parsed:?} instead of {value} for {bytes:?}"
            );
        }
    }

    #[test]
    fn test_varint() {
        let tests: [(Vec<byte>, i32); 11] = [
            (vec![0x00], 0),
            (vec![0x01], 1),
            (vec![0x02], 2),
            (vec![0x7f], 127),
            (vec![0x80, 0x01], 128),
            (vec![0xff, 0x01], 255),
            (vec![0xdd, 0xc7, 0x01], 25565),
            (vec![0xff, 0xff, 0x7f], 2_097_151),
            (vec![0xff, 0xff, 0xff, 0xff, 0x07], 2_147_483_647),
            (vec![0xff, 0xff, 0xff, 0xff, 0x0f], -1),
            (vec![0x80, 0x80, 0x80, 0x80, 0x08], -2_147_483_648),
        ];
        for (bytes, value) in tests {
            // Value -> Bytes
            let buf: Vec<u8> = VarInt::from(value).into();
            assert_eq!(buf, bytes);

            // Bytes -> Value
            assert_eq!(
                VarInt::try_from(&mut VecDeque::from(bytes)),
                Ok(VarInt(value))
            );
        }
    }

    #[test]
    fn test_varlong() {
        let tests: [(Vec<byte>, i64); 11] = [
            (vec![0x00], 0),
            (vec![0x01], 1),
            (vec![0x02], 2),
            (vec![0x7f], 127),
            (vec![0x80, 0x01], 128),
            (vec![0xff, 0x01], 255),
            (vec![0xff, 0xff, 0xff, 0xff, 0x07], 2_147_483_647),
            (
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
                9_223_372_036_854_775_807,
            ),
            (
                vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
                -1,
            ),
            (
                vec![0x80, 0x80, 0x80, 0x80, 0xf8, 0xff, 0xff, 0xff, 0xff, 0x01],
                -2_147_483_648,
            ),
            (
                vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
                -9_223_372_036_854_775_808,
            ),
        ];
        for (bytes, value) in tests {
            // Value -> Bytes
            let buf: Vec<u8> = VarLong::from(value).into();
            assert_eq!(buf, bytes);

            // Bytes -> Value
            assert_eq!(
                VarLong::try_from(&mut VecDeque::from(bytes)),
                Ok(VarLong(value))
            );
        }
    }

    #[test]
    fn debug() {
        let buf: Vec<u8> = vec![
            135, 0, 0, 0, 0, 0, 0, 2, 123, 34, 118, 101, 114, 115, 105, 111, 110, 34, 58,
            123, 34, 110, 97, 109, 101, 34, 58, 34, 49, 46, 50, 49, 46, 50, 34, 44, 34,
            112, 114, 111, 116, 111, 99, 111, 108, 34, 58, 55, 54, 56, 125, 44, 34, 112,
            108, 97, 121, 101, 114, 115, 34, 58, 123, 34, 109, 97, 120, 34, 58, 49, 48,
            48, 44, 34, 111, 110, 108, 105, 110, 101, 34, 58, 53, 44, 34, 115, 97, 109,
            112, 108, 101, 34, 58, 91, 123, 34, 110, 97, 109, 101, 34, 58, 34, 116, 104,
            105, 110, 107, 111, 102, 100, 101, 97, 116, 104, 34, 44, 34, 105, 100, 34,
            58, 34, 52, 53, 54, 54, 101, 54, 57, 102, 45, 99, 57, 48, 55, 45, 52, 56,
            101, 101, 45, 56, 100, 55, 49, 45, 100, 55, 98, 97, 53, 97, 97, 48, 48, 100,
            50, 48, 34, 125, 93, 125, 44, 34, 100, 101, 115, 99, 114, 105, 112, 116, 105,
            111, 110, 34, 58, 123, 34, 116, 101, 120, 116, 34, 58, 34, 72, 101, 108, 108,
            111, 44, 32, 119, 111, 114, 108, 100, 33, 34, 125, 44, 34, 102, 97, 118, 105,
            99, 111, 110, 34, 58, 34, 100, 97, 116, 97, 58, 105, 109, 97, 103, 101, 47,
            112, 110, 103, 59, 98, 97, 115, 101, 54, 52, 44, 60, 100, 97, 116, 97, 62,
            34, 44, 34, 101, 110, 102, 111, 114, 99, 101, 115, 83, 101, 99, 117, 114,
            101, 67, 104, 97, 116, 34, 58, 102, 97, 108, 115, 101, 125,
        ];
        println!("{:#?}", VarString::try_from(&mut VecDeque::from(buf)));
        let buf2: Vec<u8> = VarString::from("{\"version\":{\"name\":\"1.21.2\",\"protocol\":768},\"players\":{\"max\":100,\"online\":5,\"sample\":[{\"name\":\"thinkofdeath\",\"id\":\"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"}]},\"description\":{\"text\":\"Hello, world!\"},\"favicon\":\"data:image/png;base64,<data>\",\"enforcesSecureChat\":false}".to_string()).into();
        println!("{buf2:?}");

        println!(
            "Length is really {:#?}",
            VarInt::try_from(&mut VecDeque::from(vec![135, 0, 0, 0, 0, 0, 0, 2]))
        );
    }
}
