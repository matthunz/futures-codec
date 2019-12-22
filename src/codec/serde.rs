use std::io;
use std::marker::PhantomData;
use bytes::{BytesMut, Bytes};
use serde::Serialize;
use serde::de::DeserializeOwned;
use bincode;

use crate::encoder::Encoder;
use crate::decoder::Decoder;
use crate::codec::LengthCodec;

/// Encodes/decodes types implementing Serde Serialize/Deserialize traits.
/// It is built on top of `LengthCodec`.
pub struct SerdeCodec<T: Serialize + DeserializeOwned> {
    inner: LengthCodec,
    phantom: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> Default for SerdeCodec<T> {
    fn default() -> Self {
        Self {
            inner: LengthCodec {},
            phantom: PhantomData,
        }
    }
}

impl<T: Serialize + DeserializeOwned> Encoder for SerdeCodec<T> {
     type Item = T;
     type Error = io::Error;

     fn encode(&mut self, src: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
         let data = bincode::serialize(&src).map_err(to_io_err)?;
         let bytes = Bytes::from(data);
         self.inner.encode(bytes, dst)
     }
}

impl<T: Serialize + DeserializeOwned> Decoder for SerdeCodec<T> {
     type Item = T;
     type Error = io::Error;

     fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
         match self.inner.decode(src)? {
             Some(bytes) => {
                 bincode::deserialize(&bytes)
                     .map_err(to_io_err)
                     .map(|item| Some(item))
            }
             None => Ok(None),
         }
     }
}

fn to_io_err(err: Box<bincode::ErrorKind>) -> io::Error {
    match *err {
        bincode::ErrorKind::Io(e) => e,
        other => io::Error::new(io::ErrorKind::Other, other),
    }
}
