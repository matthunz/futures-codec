use crate::{Decoder, Encoder};
use std::io::Error;

pub struct BytesCodec {}

impl Decoder for BytesCodec {
    type Item = ();
    type Error = Error;
}

impl Encoder for BytesCodec {}

#[cfg(test)]
mod tests {
    use super::BytesCodec;
    use crate::Framed;
    use std::io::Cursor;
    use futures::{executor, TryStreamExt};

    #[test]
    fn decodes() {
        let mut buf = [0u8; 32];
        let mut cur = Cursor::new(&mut buf);
        let mut framed = Framed::new(cur, BytesCodec {});

        executor::block_on(framed.try_next()).unwrap().unwrap();
    }
}