use std::{fmt::LowerHex, num::ParseIntError, string::FromUtf8Error};

pub mod pad;

#[derive(Debug)]
pub enum HexToBytesError {
    ParseError(ParseIntError),
    UTF8Error(FromUtf8Error),
}
impl From<ParseIntError> for HexToBytesError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseError(error)
    }
}
impl From<FromUtf8Error> for HexToBytesError {
    fn from(error: FromUtf8Error) -> Self {
        Self::UTF8Error(error)
    }
}

/**
Parses a number into a [byte](u8) vector where each byte holds the value of a hex-pair from the
input.
*/
#[trait_variant::make(Send)]
pub trait HexToBytes: LowerHex {
    async fn hex_to_bytes(&self) -> Vec<u8>;
}
impl<T: ?Sized + Send + Sync + LowerHex> HexToBytes for T {
    async fn hex_to_bytes(&self) -> Vec<u8> {
        let s = {
            let mut r = format!("{:x}", self);
            if r.len() % 2 == 1 {
                r = "0".to_owned() + &r
            }
            r
        };

        let mut r = Vec::new();
        for chunk in s.as_bytes().chunks(2) {
            r.push(u8::from_str_radix(&String::from_utf8(chunk.to_vec()).unwrap(), 16).unwrap())
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::{pad::*, HexToBytes};

    #[tokio::test]
    async fn left_padded() {
        let expected = [&[0u8; 31], b"A" as &[u8]].concat();
        let actual = b"A".pad_left::<32>().await.to_vec();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn hexbytes() {
        assert_eq!(
            0x10203040u32.hex_to_bytes().await,
            &[0x10u8, 0x20u8, 0x30u8, 0x40u8]
        );
    }

    #[tokio::test]
    async fn right_padded_hexbytes() {
        assert_eq!(
            &(0x12030.hex_to_bytes().await.pad_right::<4>().await) as &[u8],
            &[0x01, 0x20, 0x30, 0]
        )
    }
}
