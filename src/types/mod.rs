use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    num::TryFromIntError,
    string::FromUtf8Error,
};

use thiserror::Error;

pub mod macros;
mod test;
pub mod var;

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

#[derive(Error, Debug)]
pub enum DataTypeDecodeError {
    #[error("VarNumber ended prematurely: {0:X?}")]
    PrematureEndOfVarNumber(VecDeque<u8>),

    #[error("VarNumber too big")]
    VarNumberTooBig,

    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),

    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Premature end of stream while reading a fixed length data type")]
    PrematureEnd,

    #[error("Invalid VarInt Enum variant: {variant} for {enumeration}")]
    InvalidVarIntEnumVariant {
        variant: var::VarInt,
        enumeration: String,
    },

    #[error(transparent)]
    IOError(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum DataTypeEncodeError {
    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),

    #[error(transparent)]
    IOError(#[from] io::Error),
}

#[allow(dead_code)] // I swear this is useful
pub trait DataType<Inner>: Clone {
    fn new(value: Inner) -> Self;

    fn get(&self) -> Inner;

    fn get_ref(&self) -> &Inner;

    fn decode(from: &mut impl Read) -> Result<Self, DataTypeDecodeError>;

    fn encode(&self, to: &mut impl Write) -> Result<(), DataTypeEncodeError>;
}
