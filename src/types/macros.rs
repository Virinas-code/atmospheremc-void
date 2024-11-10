//! Macros for implementing [`DataType`].
use std::{
    io::{Read, Write},
    mem::size_of,
};

use super::{DataType, DataTypeDecodeError, DataTypeEncodeError};

/// Add an implementation of [`DataType`] for primitives.
macro_rules! add_impl {
    ($($t:ty)*) => {$(
        impl DataType<$t> for $t {
            fn new(value: $t) -> Self {
                value
            }

            fn get(&self) -> $t {
                *self
            }

            fn get_ref(&self) -> &$t {
                self
            }

            fn decode(from: &mut impl Read) -> Result<Self, DataTypeDecodeError> {
                let mut buf: [u8; size_of::<$t>()] = [0; size_of::<$t>()];
                from.read_exact(&mut buf)?;

                Ok(<$t>::from_be_bytes(
                    buf,
                ))
            }

            fn encode(&self, to: &mut impl Write) -> Result<(), DataTypeEncodeError> {
                to.write_all(&self.to_be_bytes())?;
                Ok(())
            }
        }
    )*};
}

add_impl!(u8 u16 i64);

/// Add an implementation of [`DataType::new`], [`DataType::get`] and
/// [`DataType::get_ref`] for tuple structs.
#[macro_export]
macro_rules! add_tuple_impl {
    ($i:ident $t:ty) => {
        fn new(value: $t) -> Self {
            Self(value)
        }

        fn get(&self) -> $t {
            self.0
        }

        fn get_ref(&self) -> &$t {
            &self.0
        }
    };
}

// /// Read a single byte from a [`Read`] object, returning a <code>[Result]\<[u8],
// /// [DataTypeDecodeError]></code>.
// #[macro_export]
// macro_rules! read_byte {
//     ($from:ident) => {{
//         let mut buf: [u8; 1] = [0; 1];
//         $from.read_exact(&mut buf)?;
//         let out: Result<u8, DataTypeDecodeError> = Ok(buf[0]);
//         out
//     }};
// }

// /// Read N bytes from a [`Read`] object, returning a <code>[Result]\<[u8],
// /// [DataTypeDecodeError]></code>.
// #[macro_export]
// macro_rules! read_bytes {
//     ($from:ident, $length:ident) => {{
//         let mut buf: Vec<u8> = vec![0; $length];
//         $from.read_exact(&mut buf)?;
//         let out: Result<Vec<u8>, DataTypeDecodeError> = Ok(buf);
//         out
//     }};
// }
