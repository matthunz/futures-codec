use super::framed::Fuse;
use super::Encoder;
use bytes::BytesMut;
use futures::io::{AsyncRead, AsyncWrite};
use futures::{ready, Sink};
use std::io::{Error, ErrorKind};
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A `Sink` of frames encoded to an `AsyncWrite`.
///
/// # Example
/// ```
/// #![feature(async_await)]
/// use bytes::Bytes;
/// use futures_codec::{FramedWrite, BytesCodec};
/// use futures::{executor, SinkExt};
///
/// executor::block_on(async move {
///     let mut buf = Vec::new();
///     let mut framed = FramedWrite::new(&mut buf, BytesCodec {});
///
///     let msg = Bytes::from("Hello World!");
///     framed.send(msg.clone()).await.unwrap();
///
///     assert_eq!(&buf[..], &msg[..]);
/// })
/// ```
pub struct FramedWrite<T, E> {
    inner: FramedWrite2<Fuse<T, E>>,
}

impl<T, E> FramedWrite<T, E>
where
    T: AsyncWrite,
    E: Encoder,
{
    /// Creates a new `FramedWrite` transport with the given `Encoder`.
    pub fn new(inner: T, encoder: E) -> Self {
        Self {
            inner: framed_write_2(Fuse(inner, encoder)),
        }
    }

    /// High-water mark for writes, in bytes
    ///
    /// The send *high-water mark* prevents the `FramedWrite`
    /// from accepting additional messages to send when its
    /// buffer exceeds this length, in bytes. Attempts to enqueue
    /// additional messages will be deferred until progress is
    /// made on the underlying `AsyncWrite`. This applies
    /// back-pressure on fast senders and prevents unbounded
    /// buffer growth.
    ///
    /// See [`set_send_high_water_mark()`](#method.set_send_high_water_mark).
    pub fn send_high_water_mark(&self) -> usize {
        return self.inner.high_water_mark;
    }

    /// Sets high-water mark for writes, in bytes
    ///
    /// The send *high-water mark* prevents the `FramedWrite`
    /// from accepting additional messages to send when its
    /// buffer exceeds this length, in bytes. Attempts to enqueue
    /// additional messages will be deferred until progress is
    /// made on the underlying `AsyncWrite`. This applies
    /// back-pressure on fast senders and prevents unbounded
    /// buffer growth.
    ///
    /// The default high-water mark is 2^17 bytes. Applications
    /// which desire low latency may wish to reduce this value.
    /// There is little point to increasing this value beyond
    /// your socket's `SO_SNDBUF` size. On linux, this defaults
    /// to 212992 bytes but is user-adjustable.
    pub fn set_send_high_water_mark(&mut self, hwm: usize) {
        self.inner.high_water_mark = hwm;
    }

    /// Release the I/O and Encoder
    pub fn release(self) -> (T, E) {
        let fuse = self.inner.release();
        (fuse.0, fuse.1)
    }
}

impl<T, E> Sink<E::Item> for FramedWrite<T, E>
where
    T: AsyncWrite + Unpin,
    E: Encoder,
{
    type Error = E::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_ready(cx)
    }
    fn start_send(mut self: Pin<&mut Self>, item: E::Item) -> Result<(), Self::Error> {
        Pin::new(&mut self.inner).start_send(item)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

pub struct FramedWrite2<T> {
    pub inner: T,
    pub high_water_mark: usize,
    buffer: BytesMut,
}

// 2^17 bytes, which is slightly over 60% of the default
// TCP send buffer size (SO_SNDBUF)
const DEFAULT_SEND_HIGH_WATER_MARK: usize = 131072;

pub fn framed_write_2<T>(inner: T) -> FramedWrite2<T> {
    FramedWrite2 {
        inner,
        high_water_mark: DEFAULT_SEND_HIGH_WATER_MARK,
        buffer: BytesMut::with_capacity(1028 * 8),
    }
}

impl<T> Unpin for FramedWrite2<T> {}

impl<T: AsyncRead + Unpin> AsyncRead for FramedWrite2<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<T> Sink<T::Item> for FramedWrite2<T>
where
    T: AsyncWrite + Encoder + Unpin,
{
    type Error = T::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let this = &mut *self;
        while this.buffer.len() >= this.high_water_mark {
            let num_write = ready!(Pin::new(&mut this.inner).poll_write(cx, &this.buffer))?;

            if num_write == 0 {
                return Poll::Ready(Err(err_eof().into()));
            }

            this.buffer.advance(num_write);
        }

        Poll::Ready(Ok(()))
    }
    fn start_send(mut self: Pin<&mut Self>, item: T::Item) -> Result<(), Self::Error> {
        let this = &mut *self;
        this.inner.encode(item, &mut this.buffer)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let this = &mut *self;

        while !this.buffer.is_empty() {
            let num_write = ready!(Pin::new(&mut this.inner).poll_write(cx, &this.buffer))?;

            if num_write == 0 {
                return Poll::Ready(Err(err_eof().into()));
            }

            this.buffer.advance(num_write);
        }

        Pin::new(&mut this.inner).poll_flush(cx).map_err(Into::into)
    }
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll_flush(cx))?;
        Pin::new(&mut self.inner).poll_close(cx).map_err(Into::into)
    }
}

impl<T> FramedWrite2<T> {
    pub fn release(self) -> T {
        self.inner
    }
}

fn err_eof() -> Error {
    Error::new(ErrorKind::UnexpectedEof, "End of file")
}
