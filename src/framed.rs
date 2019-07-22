use super::framed_read::{framed_read_2, FramedRead2};
use super::framed_write::{framed_write_2, FramedWrite2};
use super::{Decoder, Encoder};
use futures::io::{AsyncRead, AsyncWrite};
use futures::{Sink, Stream, TryStreamExt};
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
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        self.pinned_t().poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.pinned_t().poll_flush(cx)
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        self.pinned_t().poll_close(cx)
    }
}

/// A unified `Stream` and `Sink` interface to an underlying I/O object,
/// using the `Encoder` and `Decoder` traits to encode and decode frames.
///
/// # Example
/// ```
/// #![feature(async_await, await_macro)]
/// use bytes::Bytes;
/// use futures::{executor, SinkExt, TryStreamExt};
/// use std::io::Cursor;
/// use futures_codec::{BytesCodec, Framed};
///
/// executor::block_on(async move {
///     let cur = Cursor::new(vec![0u8; 12]);
///     let mut framed = Framed::new(cur, BytesCodec {});
///
///     // Send bytes to `buf` through the `BytesCodec`
///     let bytes = Bytes::from("Hello world!");
///     framed.send(bytes).await.unwrap();
///
///     // Dispose of the framer and return the I/O and codec
///     let (cur, _) = framed.release();
///     assert_eq!(cur.get_ref(), b"Hello world!");
/// })
/// ```
pub struct Framed<T, U> {
    inner: FramedRead2<FramedWrite2<Fuse<T, U>>>,
}

impl<T, U> Framed<T, U>
where
    T: AsyncRead + AsyncWrite,
    U: Decoder + Encoder,
{
    /// Creates a new `Framed` transport with the given codec.
    /// A codec is a type which implements `Decoder` and `Encoder`.
    pub fn new(inner: T, codec: U) -> Self {
        Self {
            inner: framed_read_2(framed_write_2(Fuse(inner, codec))),
        }
    }

    /// Release the I/O and Codec
    pub fn release(self: Self) -> (T, U) {
        let fuse = self.inner.release().release();
        (fuse.0, fuse.1)
    }
}

impl<T, U> Stream for Framed<T, U>
where
    T: AsyncRead + Unpin,
    U: Decoder,
{
    type Item = Result<U::Item, U::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.try_poll_next_unpin(cx)
    }
}

impl<T, U> Sink<U::Item> for Framed<T, U>
where
    T: AsyncWrite + Unpin,
    U: Encoder,
{
    type Error = U::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_ready(cx)
    }
    fn start_send(mut self: Pin<&mut Self>, item: U::Item) -> Result<(), Self::Error> {
        Pin::new(&mut self.inner).start_send(item)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}
