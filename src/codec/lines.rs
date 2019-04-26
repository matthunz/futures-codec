use crate::{Decoder, Encoder};
use bytes::{BufMut, BytesMut};
use std::io::{Error, ErrorKind};

/// A simple `Codec` implementation that splits up data into lines.
pub struct LinesCodec {}

impl Encoder for LinesCodec {
    type Item = String;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put(item);
        Ok(())
    }
}

impl Decoder for LinesCodec {
    type Item = String;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match src.iter().position(|b| b == &b'\n') {
            Some(pos) if !src.is_empty() => {
                let buf = src.split_to(pos + 1);
                String::from_utf8(buf.to_vec())
                    .map(Some)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))
            },
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::FramedRead;

    use futures::{executor, TryStreamExt};
    use std::io::Cursor;
    #[test]
    fn it_works() {
        let buf = "Hello\nWorld\n".to_owned();
        let cur = Cursor::new(buf);

        let mut framed = FramedRead::new(cur, LinesCodec {});
        let next = executor::block_on(framed.try_next()).unwrap().unwrap();
        assert_eq!(next, "Hello\n");
        let next = executor::block_on(framed.try_next()).unwrap().unwrap();
        assert_eq!(next, "World\n");
    }
}
