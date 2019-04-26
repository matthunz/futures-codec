mod codec;
pub use codec::BytesCodec;

mod framed;
pub use framed::Framed;

mod framed_read;
pub use framed_read::FramedRead;

mod framed_write;
pub use framed_write::FramedWrite;

use std::io::Error;
use bytes::BytesMut;

pub trait Decoder {
    type Item;
    type Error: From<Error>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>;
}

pub trait Encoder {}