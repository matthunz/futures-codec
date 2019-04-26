use super::{Decoder, Encoder};
use super::framed_read::{framed_read_2, FramedRead2};
use super::framed_write::{framed_write_2, FramedWrite2};
use futures::io::{AsyncRead, AsyncWrite};

pub struct Fuse<T, U>(pub T, pub U);

pub struct Framed<T, U> {
    inner: FramedRead2<FramedWrite2<Fuse<T, U>>>,
}

impl<T, U> Framed<T, U>
where
    T: AsyncRead + AsyncWrite,
    U: Decoder + Encoder,
{
    pub fn new(inner: T, codec: U) -> Self {
        Self { inner: framed_read_2(framed_write_2(Fuse(inner, codec))) }
    }
}