use crate::{Decoder, Encoder};
use bytes::{Bytes, BytesMut};
use std::io::Error;

/// A simple codec that ships bytes around
///
/// # Example
///
///  ```
/// #![feature(async_await)]
/// use bytes::Bytes;
/// use futures::{SinkExt, TryStreamExt};
/// use futures::io::Cursor;
/// use futures_codec::{BytesCodec, Framed};
///
/// async move {
///     let mut buf = vec![];
///     // Cursor implements AsyncRead and AsyncWrite
///     let cur = Cursor::new(&mut buf);
///     let mut framed = Framed::new(cur, BytesCodec {});
///
///     let msg = Bytes::from("Hello World!");
///     framed.send(msg).await.unwrap();
///
///     while let Some(msg) = framed.try_next().await.unwrap() {
///         println!("{:?}", msg);
///     }
/// };
/// ```
pub struct BytesCodec {}

impl Encoder for BytesCodec {
    type Item = Bytes;
    type Error = Error;

    fn encode(&mut self, src: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(&src);
        Ok(())
    }
}

impl Decoder for BytesCodec {
    type Item = Bytes;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.len();
        if len > 0 {
            Ok(Some(src.split_to(len).freeze()))
        } else {
            Ok(None)
        }
    }
}
