use super::Decoder;
use super::framed::Fuse;
use futures::io::AsyncRead;

pub struct FramedRead<T, D> {
    inner: FramedRead2<Fuse<T, D>>,
}

impl<T, D> FramedRead<T, D>
where
    T: AsyncRead,
    D: Decoder,
{
    pub fn new(inner: T, decoder: D) -> Self {
        Self {
            inner: framed_read_2(Fuse(inner, decoder)),
        }
    }
}

pub struct FramedRead2<T> {
    inner: T,
}

pub fn framed_read_2<T>(inner: T) -> FramedRead2<T> {
    FramedRead2 { inner }
}