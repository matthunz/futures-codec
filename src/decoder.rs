use std::io::Error;
use bytes::BytesMut;
use super::framed::Fuse;
use super::framed_write::FramedWrite2;

pub trait Decoder {
    type Item;
    type Error: From<Error>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>;
}

impl<T, U: Decoder> Decoder for Fuse<T, U> {
    type Item = U::Item;
    type Error = U::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.1.decode(src)
    }
}

impl<T: Decoder> Decoder for FramedWrite2<T> {
    type Item = T::Item;
    type Error = T::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.inner.decode(src)
    }
}