use super::fuse::Fuse;
use super::Encoder;
use bytes::{Buf, BytesMut};
use pin_project_lite::pin_project;
use std::io;
use std::ops::{Deref, DerefMut};

pin_project! {
    /// A `Sink` of frames encoded to an `AsyncWrite`.
    ///
    /// # Example
    /// ```
    /// use bytes::Bytes;
    /// use futures_codec::{FramedWrite, BytesCodec};
    /// use futures::SinkExt;
    ///
    /// # futures::executor::block_on(async move {
    /// let mut buf = Vec::new();
    /// let mut framed = FramedWrite::new(&mut buf, BytesCodec {});
    ///
    /// let bytes = Bytes::from("Hello World!");
    /// framed.send(bytes.clone()).await?;
    ///
    /// assert_eq!(&buf[..], &bytes[..]);
    /// # Ok::<_, std::io::Error>(())
    /// # }).unwrap();
    /// ```
    #[derive(Debug)]
    pub struct FramedWrite<T, E> {
        #[pin]
        inner: FramedWrite2<Fuse<T, E>>,
    }
}

impl<T, E> FramedWrite<T, E> {
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
        self.inner.high_water_mark
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
        (fuse.t, fuse.u)
    }

    /// Consumes the `FramedWrite`, returning its underlying I/O stream.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn into_inner(self) -> T {
        self.release().0
    }

    /// Returns a reference to the underlying encoder.
    ///
    /// Note that care should be taken to not tamper with the underlying encoder
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn encoder(&self) -> &E {
        &self.inner.u
    }

    /// Returns a mutable reference to the underlying encoder.
    ///
    /// Note that care should be taken to not tamper with the underlying encoder
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn encoder_mut(&mut self) -> &mut E {
        &mut self.inner.u
    }
}

impl<T, E> Deref for FramedWrite<T, E> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T, E> DerefMut for FramedWrite<T, E> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

pin_project! {
    #[derive(Debug)]
    pub struct FramedWrite2<T> {
        #[pin]
        pub inner: T,
        pub high_water_mark: usize,
        buffer: BytesMut,
    }
}

impl<T> Deref for FramedWrite2<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> DerefMut for FramedWrite2<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

// 2^17 bytes, which is slightly over 60% of the default
// TCP send buffer size (SO_SNDBUF)
const DEFAULT_SEND_HIGH_WATER_MARK: usize = 131072;

impl<T> FramedWrite2<T> {
    pub fn new(inner: T) -> FramedWrite2<T> {
        FramedWrite2 {
            inner,
            high_water_mark: DEFAULT_SEND_HIGH_WATER_MARK,
            buffer: BytesMut::with_capacity(1028 * 8),
        }
    }

    pub fn release(self) -> T {
        self.inner
    }
}

cfg_async! {
    use futures_sink::Sink;
    use futures_util::io::{AsyncRead, AsyncWrite};
    use futures_util::ready;
    use std::marker::Unpin;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    impl<T, E> FramedWrite<T, E>
    where
        T: AsyncWrite,
        E: Encoder,
    {
        /// Creates a new `FramedWrite` transport with the given `Encoder`.
        pub fn new(inner: T, encoder: E) -> Self {
            Self {
                inner: FramedWrite2::new(Fuse::new(inner, encoder)),
            }
        }
    }

    impl<T, E> Sink<E::Item> for FramedWrite<T, E>
    where
        T: AsyncWrite + Unpin,
        E: Encoder,
    {
        type Error = E::Error;

        fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
            self.project().inner.poll_ready(cx)
        }
        fn start_send(self: Pin<&mut Self>, item: E::Item) -> Result<(), Self::Error> {
            self.project().inner.start_send(item)
        }
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
            self.project().inner.poll_flush(cx)
        }
        fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
            self.project().inner.poll_close(cx)
        }
    }

    impl<T: AsyncRead + Unpin> AsyncRead for FramedWrite2<T> {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            self.project().inner.poll_read(cx, buf)
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
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
            let mut this = self.project();

            while !this.buffer.is_empty() {
                let num_write = ready!(Pin::new(&mut this.inner).poll_write(cx, &this.buffer))?;

                if num_write == 0 {
                    return Poll::Ready(Err(err_eof().into()));
                }

                this.buffer.advance(num_write);
            }

            this.inner.poll_flush(cx).map_err(Into::into)
        }
        fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
            ready!(self.as_mut().poll_flush(cx))?;
            self.project().inner.poll_close(cx).map_err(Into::into)
        }
    }
}

cfg_blocking! {
    use crate::sink::IterSink;

    impl<T, E> FramedWrite<T, E>
    where
        T: io::Write,
        E: Encoder,
    {
        /// Creates a new blocking `FramedWrite` transport with the given `Encoder`.
        pub fn new_blocking(inner: T, encoder: E) -> Self {
            Self {
                inner: FramedWrite2::new(Fuse::new(inner, encoder)),
            }
        }
    }

    impl<T, E> IterSink<E::Item> for FramedWrite<T, E>
    where
        T: io::Write,
        E: Encoder,
    {
        type Error = E::Error;

        fn start_send(&mut self, item: E::Item) -> Result<(), Self::Error> {
            self.inner.start_send(item)
        }

        fn ready(&mut self) -> Result<(), Self::Error> {
            self.inner.ready()
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.inner.flush()
        }
    }

    impl<T: io::Read> io::Read for FramedWrite2<T> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.inner.read(buf)
        }
    }

    impl<T> IterSink<T::Item> for FramedWrite2<T>
    where
        T: io::Write + Encoder,
    {
        type Error = T::Error;

        fn start_send(&mut self, item: T::Item) -> Result<(), Self::Error> {
            self.inner.encode(item, &mut self.buffer)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            while !self.buffer.is_empty() {
                let num_write = self.inner.write(&self.buffer)?;

                if num_write == 0 {
                    return Err(err_eof().into());
                }

                self.buffer.advance(num_write);
            }

            self.inner.flush().map_err(Into::into)
        }

        fn ready(&mut self) -> Result<(), Self::Error> {
            while self.buffer.len() >= self.high_water_mark {
                let num_write = self.inner.write(&self.buffer)?;

                if num_write == 0 {
                    return Err(err_eof().into());
                }

                self.buffer.advance(num_write);
            }
            Ok(())
        }
    }
}

fn err_eof() -> io::Error {
    io::Error::new(io::ErrorKind::UnexpectedEof, "End of file")
}
