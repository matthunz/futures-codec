use crate::Encoder;
use crate::Fuse;
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