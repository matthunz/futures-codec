use super::fuse::Fuse;
use super::Decoder;

use bytes::BytesMut;
use futures::io::AsyncRead;
use futures::{ready, Sink, Stream, TryStreamExt};
use pin_project::pin_project;
use std::io;
use std::marker::Unpin;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

/// A `Stream` of messages decoded from an `AsyncRead`.
///
/// # Example
/// ```
/// use futures_codec::{BytesCodec, FramedRead};
/// use futures::TryStreamExt;
/// use bytes::{Bytes};
///
/// let buf = [3u8; 3];
/// let mut framed = FramedRead::new(&buf[..], BytesCodec);
///
/// # futures::executor::block_on(async move {
/// if let Some(bytes) = framed.try_next().await? {
///     assert_eq!(bytes, Bytes::copy_from_slice(&buf[..]));
/// }
/// # Ok::<_, std::io::Error>(())
/// # }).unwrap();
/// ```
#[derive(Debug)]
pub struct FramedRead<T, D> {
    inner: FramedRead2<Fuse<T, D>>,
}

impl<T, D> Deref for FramedRead<T, D> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T, D> DerefMut for FramedRead<T, D> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T, D> FramedRead<T, D>
where
    T: AsyncRead,
    D: Decoder,
{
    /// Creates a new `FramedRead` transport with the given `Decoder`.
    pub fn new(inner: T, decoder: D) -> Self {
        Self {
            inner: framed_read_2(Fuse::new(inner, decoder)),
        }
    }

    /// Release the I/O and Decoder
    pub fn release(self: Self) -> (T, D) {
        let fuse = self.inner.release();
        (fuse.t, fuse.u)
    }

    /// Consumes the `FramedRead`, returning its underlying I/O stream.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn into_inner(self) -> T {
        self.release().0
    }

    /// Returns a reference to the underlying decoder.
    ///
    /// Note that care should be taken to not tamper with the underlying decoder
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn decoder(&self) -> &D {
        &self.inner.u
    }

    /// Returns a mutable reference to the underlying decoder.
    ///
    /// Note that care should be taken to not tamper with the underlying decoder
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn decoder_mut(&mut self) -> &mut D {
        &mut self.inner.u
    }

    /// Returns a reference to the read buffer.
    pub fn read_buffer(&self) -> &BytesMut {
        &self.inner.buffer
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

#[pin_project]
#[derive(Debug)]
pub struct FramedRead2<T> {
    #[pin]
    inner: T,
    buffer: BytesMut,
}

impl<T> Deref for FramedRead2<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> DerefMut for FramedRead2<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
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

            let ended = n == 0;

            match this.inner.decode(&mut this.buffer)? {
                Some(item) => return Poll::Ready(Some(Ok(item))),
                None if ended => {
                    if this.buffer.is_empty() {
                        return Poll::Ready(None);
                    } else {
                        match this.inner.decode_eof(&mut this.buffer)? {
                            Some(item) => return Poll::Ready(Some(Ok(item))),
                            None if this.buffer.is_empty() => return Poll::Ready(None),
                            None => {
                                return Poll::Ready(Some(Err(io::Error::new(
                                    io::ErrorKind::UnexpectedEof,
                                    "bytes remaining in stream",
                                )
                                .into())));
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
    }
}

impl<T, I> Sink<I> for FramedRead2<T>
where
    T: Sink<I> + Unpin,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }
    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        self.project().inner.start_send(item)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

impl<T> FramedRead2<T> {
    pub fn release(self: Self) -> T {
        self.inner
    }

    pub fn buffer(&self) -> &BytesMut {
        &self.buffer
    }
}
