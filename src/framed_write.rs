use super::{Decoder, Encoder};
use super::framed::Fuse;
use futures::io::AsyncWrite;
use futures::TryStream;
use futures::io::AsyncRead;
use std::io::Error;
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

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

impl<T> Unpin for FramedWrite2<T> {}

impl<T: AsyncRead + Unpin> AsyncRead for FramedWrite2<T> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize, Error>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<T: Decoder> Decoder for FramedWrite2<T> {
    type Item = T::Item;
    type Error = T::Error;
}