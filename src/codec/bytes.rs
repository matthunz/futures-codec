use crate::{Decoder, Encoder};

pub struct BytesCodec {}

impl Decoder for BytesCodec {}

impl Encoder for BytesCodec {}

#[cfg(test)]
mod tests {
    use super::BytesCodec;
    use crate::Framed;
    use std::io::Cursor;

    #[test]
    fn decodes_and_encodes() {
        let mut buf = [0u8; 32];
        let mut cur = Cursor::new(&mut buf);
        let _framed = Framed::new(cur, BytesCodec {});
    }
}