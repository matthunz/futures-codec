use crate::{Decoder, Encoder};
use bytes::{Bytes, BytesMut};
use std::io::Error;

pub struct BytesCodec {}

impl Decoder for BytesCodec {
    type Item = Bytes;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.len();
        Ok(Some(src.split_to(len).freeze()))
    }
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

        let read = executor::block_on(framed.try_next()).unwrap().unwrap();
        assert_eq!(&read[..], &buf[..]);
    }
}