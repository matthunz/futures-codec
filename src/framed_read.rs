use super::framed::Fuse;
use super::Decoder;

use bytes::BytesMut;
use futures::io::AsyncRead;
use futures::{ready, Sink, Stream, TryStreamExt};
use std::io;
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A `Stream` of messages decoded from an `AsyncRead`.
///
/// # Example
/// ```
/// #![feature(async_await, await_macro)]
/// use futures_codec::{BytesCodec, FramedRead};
/// use futures::{executor, TryStreamExt};
/// use bytes::Bytes;
///
/// let buf = b"Hello World!";
/// let mut framed = FramedRead::new(&buf[..], BytesCodec {});
///
/// executor::block_on(async move {
///     let msg = framed.try_next().await.unwrap().unwrap();
///     assert_eq!(msg, Bytes::from(&buf[..]));
/// })
/// ```
pub struct FramedRead<T, D> {
    inner: FramedRead2<Fuse<T, D>>,
}

impl<T, D> FramedRead<T, D>
where
    T: AsyncRead,
    D: Decoder,
{
    /// Creates a new `FramedRead` transport with the given `Decoder`.
    pub fn new(inner: T, decoder: D) -> Self {
        Self {
            inner: framed_read_2(Fuse(inner, decoder)),
        }
    }

    /// Release the I/O and Decoder
    pub fn release(self: Self) -> (T, D) {
        let fuse = self.inner.release();
        (fuse.0, fuse.1)
    }
}

impl<T, D> Stream for FramedRead<T, D>
where
    T: AsyncRead + Unpin,
    D: Decoder,
{
    type Item = Result<D::Item, D::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.try_poll_next_unpin(cx)
    }
}

pub struct FramedRead2<T> {
    inner: T,
    buffer: BytesMut,
}

const INITIAL_CAPACITY: usize = 8 * 1024;

pub fn framed_read_2<T>(inner: T) -> FramedRead2<T> {
    FramedRead2 {
        inner,
        buffer: BytesMut::with_capacity(INITIAL_CAPACITY),
    }
}

impl<T> Stream for FramedRead2<T>
where
    T: AsyncRead + Decoder + Unpin,
{
    type Item = Result<T::Item, T::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;

        if let Some(item) = this.inner.decode(&mut this.buffer)? {
            return Poll::Ready(Some(Ok(item)));
        }

        let mut buf = [0u8; INITIAL_CAPACITY];

        loop {
            let n = ready!(Pin::new(&mut this.inner).poll_read(cx, &mut buf))?;
            this.buffer.extend_from_slice(&buf[..n]);

            match this.inner.decode(&mut this.buffer)? {
                Some(item) => return Poll::Ready(Some(Ok(item))),
                None => {
                    if this.buffer.is_empty() {
                        return Poll::Ready(None);
                    } else if n == 0 {
                        return Poll::Ready(Some(Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "bytes remaining in stream",
                        )
                        .into())));
                    }
                }
            }
        }
    }
}

impl<T, I> Sink<I> for FramedRead2<T>
where
    T: Sink<I> + Unpin,
{
    type Error = T::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_ready(cx)
    }
    fn start_send(mut self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        Pin::new(&mut self.inner).start_send(item)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

impl<T> FramedRead2<T> {
    pub fn release(self: Self) -> T {
        self.inner
    }
}
