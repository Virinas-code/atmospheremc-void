use std::{collections::VecDeque, mem::size_of};

use super::{drain, DataType, DataTypeDecodeError, DataTypeEncodeError};

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

            fn decode(value: &mut VecDeque<u8>) -> Result<Self, DataTypeDecodeError> {
                Ok(<$t>::from_be_bytes(
                    drain(value, 0..size_of::<$t>())?
                        .try_into()
                        .map_err(|_| DataTypeDecodeError::PrematureEnd)?,
                ))
            }

            fn encode(&self) -> Result<Vec<u8>, DataTypeEncodeError> {
                Ok(Vec::from(self.to_be_bytes()))
            }
        }
    )*};
}

add_impl!(u8 u16 i64);

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
