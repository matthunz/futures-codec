use super::Decoder;
use super::framed::Fuse;
use futures::TryStream;
use futures::io::AsyncRead;
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

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

impl<T> TryStream for FramedRead2<T>
where
    T: AsyncRead + Decoder + Unpin,
{
    type Ok = T::Item;
    type Error = T::Error;

    fn try_poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Ok, Self::Error>>> {
        let mut buf = [0u8; 32];
        Pin::new(&mut self.inner).poll_read(cx, &mut buf);
        unimplemented!()
    }
}