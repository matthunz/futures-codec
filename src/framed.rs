use super::framed_read::{framed_read_2, FramedRead2};
use super::framed_write::{framed_write_2, FramedWrite2};
use super::{Decoder, Encoder};
use futures::TryStream;
use futures::io::{AsyncRead, AsyncWrite};
use std::io::Error;
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Fuse<T, U>(pub T, pub U);

impl<T: Unpin, U> Fuse<T, U> {
    pub fn pinned_t<'a>(self: Pin<&'a mut Self>) -> Pin<&'a mut T> {
        Pin::new(&mut self.get_mut().0)
    }
}

impl<T, U> Unpin for Fuse<T, U> {}

impl<T: AsyncRead + Unpin, U> AsyncRead for Fuse<T, U> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        self.pinned_t().poll_read(cx, buf)
    }
}

impl<T: AsyncWrite + Unpin, U> AsyncWrite for Fuse<T, U> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        self.pinned_t().poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.pinned_t().poll_flush(cx)
    }
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.pinned_t().poll_close(cx)
    }
}

pub struct Framed<T, U> {
    inner: FramedRead2<FramedWrite2<Fuse<T, U>>>,
}

impl<T, U> Framed<T, U>
where
    T: AsyncRead + AsyncWrite,
    U: Decoder + Encoder,
{
    pub fn new(inner: T, codec: U) -> Self {
        Self {
            inner: framed_read_2(framed_write_2(Fuse(inner, codec))),
        }
    }
}

impl<T, U> TryStream for Framed<T, U>
where
    T: AsyncRead + Unpin,
    U: Decoder,
{
    type Ok = U::Item;
    type Error = U::Error;

    fn try_poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Ok, Self::Error>>> {
        Pin::new(&mut self.inner).try_poll_next(cx)
    }
}
