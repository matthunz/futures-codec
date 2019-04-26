use bytes::BytesMut;
use std::io::Error;
use super::framed::Fuse;

pub trait Encoder {
    type Item;
    type Error: From<Error>;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error>;
}

impl<T, U: Encoder> Encoder for Fuse<T, U> {
    type Item = U::Item;
    type Error = U::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.1.encode(item, dst)
    }
}