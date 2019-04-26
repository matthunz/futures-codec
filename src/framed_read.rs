use crate::Decoder;
use crate::Fuse;
use futures::io::AsyncRead;

pub struct FramedRead<T, D> {
    fused: Fuse<T, D>,
}

impl<T, D> FramedRead<T, D>
where
    T: AsyncRead,
    D: Decoder,
{
    pub fn new(inner: T, decoder: D) -> Self {
        Self {
            fused: Fuse(inner, decoder),
        }
    }
}