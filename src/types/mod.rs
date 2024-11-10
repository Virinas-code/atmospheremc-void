//! Types used in the protocol.
use std::{
    cmp,
    collections::VecDeque,
    io::{self, Read, Write},
    num::TryFromIntError,
    string::FromUtf8Error,
};

use thiserror::Error;

pub mod macros;
mod test;
pub mod var;

/// Chunk size to read in [`ReadBytes::read_bytes`].
const CHUNK_SIZE: usize = 256;

// fn drain<R>(bytes: &impl Read, range: R) -> Result<Vec<u8>, DataTypeDecodeError>
// where
//     R: RangeBounds<usize> + Iterator,
// {
//     let mut output: Vec<u8> = Vec::new();
//     for _ in range {
//         output.push(bytes.rea().ok_or_else(|| {
//             DataTypeDecodeError::PrematureEndOfVarNumber(bytes.clone())
//         })?);
//     }
//     Ok(output)
// }

/// Error when decoding a [`DataType`] using [`DataType::decode`].
#[derive(Error, Debug)]
pub enum DataTypeDecodeError {
    /// Stream ended prematurely while reading variable length data ([`var`]).
    #[error("VarNumber ended prematurely: {0:X?}")]
    PrematureEndOfVarNumber(VecDeque<u8>),

    /// Variable length data ([`var`]) exceeds maximum value.
    #[error("VarNumber too big")]
    VarNumberTooBig,

    /// Error when converting a type from an integer. See [`TryFromIntError`].
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),

    /// Error when converting a type from UTF-8. See [`FromUtf8Error`].
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),

    /// Stream ended prematurely while reading fixed length data.
    #[error("Premature end of stream while reading a fixed length data type")]
    PrematureEnd,

    /// When reading an enum, an invalid variant is received.
    #[error("Invalid VarInt Enum variant: {variant} for {enumeration}")]
    InvalidVarIntEnumVariant {
        /// The variant received.
        variant: var::VarInt,

        /// The name of the enum we tried to read.
        enumeration: String,
    },

    /// An [`io::Error`].
    #[error(transparent)]
    IOError(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum DataTypeEncodeError {
    /// Error when converting a type from an integer. See [`TryFromIntError`].
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),

    /// An [`io::Error`].
    #[error(transparent)]
    IOError(#[from] io::Error),
}

pub trait DataType<Inner>: Clone {
    /// Create a new instance of the data type from a value.
    fn new(value: Inner) -> Self;

    /// Get the value.
    #[allow(dead_code)]
    fn get(&self) -> Inner;

    /// Get the value as a reference.
    #[allow(dead_code)]
    fn get_ref(&self) -> &Inner;

    /// Decode the value from a [`Read`] stream.
    fn decode(from: &mut impl Read) -> Result<Self, DataTypeDecodeError>;

    /// Encode the value to a [`Write`] stream.
    fn encode(&self, to: &mut impl Write) -> Result<(), DataTypeEncodeError>;
}

/// A helper trait for reading bytes easily:
///
/// ```
/// # let buf: Vec<u8> = [0x01, 0x23, 0x45, 0x67, 0x89];
/// println!("Reading a single byte: {:X?}", buf.read_byte());
/// println!("Reading 4 bytes: {:X?}", buf.read_bytes(4));
/// ```
pub trait ReadBytes: Read {
    /// Read a single byte.
    fn read_byte(&mut self) -> Result<u8, io::Error> {
        let mut buf: [u8; 1] = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Read `length` bytes.
    fn read_bytes<L: Into<usize> + Copy>(
        &mut self,
        length: L,
    ) -> Result<Vec<u8>, io::Error> {
        let length = length.into();
        let mut buf: Vec<u8> = Vec::reser(length);
        let mut bytes_read: usize = 0;
        while bytes_read < length {
            let bytes_to_read: usize = cmp::min(length - bytes_read, CHUNK_SIZE);

            let mut tmp_buf: Vec<u8> = vec![0; bytes_to_read];
            self.read_exact(&mut tmp_buf)?;
            buf.append(&mut tmp_buf);

            bytes_read += bytes_to_read;
        }
        Ok(buf)
    }
}

impl<T: Read> ReadBytes for T {}
