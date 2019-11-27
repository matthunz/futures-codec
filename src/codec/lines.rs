use crate::{Decoder, Encoder};
use bytes::{BufMut, BytesMut};
use std::io::{Error, ErrorKind};
use memchr::memchr;

/// A simple `Codec` implementation that splits up data into lines.
///
/// ```rust
/// use futures::executor;
/// use futures::stream::StreamExt; // for lines.next()
/// use futures_codec::{FramedRead, LinesCodec};
///
/// let input = "hello\nworld\nthis\nis\ndog\n".as_bytes();
/// let mut lines = FramedRead::new(input, LinesCodec {});
/// executor::block_on(async move {
///     while let Some(line) = lines.next().await {
///         println!("{}", line.unwrap());
///     }
/// });
/// ```
pub struct LinesCodec {}

impl Encoder for LinesCodec {
    type Item = String;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.len());
        dst.put(item);
        Ok(())
    }
}

impl Decoder for LinesCodec {
    type Item = String;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match memchr(b'\n', src) {
            Some(pos) => {
                let buf = src.split_to(pos + 1);
                String::from_utf8(buf.to_vec())
                    .map(Some)
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))
            }
            _ => Ok(None),
        }
    }
}
