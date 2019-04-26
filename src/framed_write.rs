use super::Encoder;
use super::framed::Fuse;
use futures::io::AsyncWrite;

pub struct FramedWrite<T, E> {
    fused: Fuse<T, E>,
}

impl<T, E> FramedWrite<T, E>
where
    T: AsyncWrite,
    E: Encoder,
{
    pub fn new(inner: T, encoder: E) -> Self {
        Self {
            fused: Fuse(inner, encoder),
        }
    }
}

pub struct FramedWrite2<T> {
    inner: T,
}

pub fn framed_write_2<T>(inner: T) -> FramedWrite2<T> {
    FramedWrite2 { inner }
}