#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::types::{
        var::{VarInt, VarLong},
        DataType, DataTypeDecodeError,
    };

    #[test]
    fn test_fixed_long() {
        let tests: [(Vec<u8>, i64); 1] = [(
            vec![0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            9_223_372_036_854_775_807,
        )];
        for (bytes, value) in tests {
            let parsed: Result<i64, DataTypeDecodeError> =
                i64::decode(&mut VecDeque::from(bytes.clone()));
            assert_eq!(
                *parsed.as_ref().unwrap(),
                value,
                "Parse returned {parsed:?} instead of {value} for {bytes:?}"
            );
        }
    }

    #[test]
    fn test_varint() {
        let tests: [(Vec<u8>, i32); 11] = [
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
            let mut buf: Vec<u8> = Vec::new();
            VarInt::new(value)
                .encode(&mut buf)
                .expect("Encoding failed");
            assert_eq!(buf, bytes);

            // Bytes -> Value
            assert_eq!(
                VarInt::decode(&mut VecDeque::from(bytes)).unwrap(),
                VarInt(value)
            );
        }
    }

    #[test]
    fn test_varlong() {
        let tests: [(Vec<u8>, i64); 11] = [
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
            let mut buf: Vec<u8> = Vec::new();
            VarLong::new(value)
                .encode(&mut buf)
                .expect("Encoding failed");
            assert_eq!(buf, bytes);

            // Bytes -> Value
            assert_eq!(
                VarLong::decode(&mut VecDeque::from(bytes)).unwrap(),
                VarLong::new(value)
            );
        }
    }
}
