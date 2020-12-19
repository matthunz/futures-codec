use super::framed_read::FramedRead2;
use super::framed_write::FramedWrite2;
use super::fuse::Fuse;
use super::{Decoder, Encoder};
use bytes::BytesMut;
use pin_project_lite::pin_project;
use std::ops::{Deref, DerefMut};

pin_project! {
    /// A unified `Stream` and `Sink` interface to an underlying I/O object,
    /// using the `Encoder` and `Decoder` traits to encode and decode frames.
    ///
    /// # Example
    /// ```
    /// use bytes::Bytes;
    /// use futures::{SinkExt, TryStreamExt};
    /// use futures::io::Cursor;
    /// use futures_codec::{BytesCodec, Framed};
    ///
    /// # futures::executor::block_on(async move {
    /// let cur = Cursor::new(vec![0u8; 12]);
    /// let mut framed = Framed::new(cur, BytesCodec {});
    ///
    /// // Send bytes to `buf` through the `BytesCodec`
    /// let bytes = Bytes::from("Hello world!");
    /// framed.send(bytes).await?;
    ///
    /// // Release the I/O and codec
    /// let (cur, _) = framed.release();
    /// assert_eq!(cur.get_ref(), b"Hello world!");
    /// # Ok::<_, std::io::Error>(())
    /// # }).unwrap();
    /// ```
    #[derive(Debug)]
    pub struct Framed<T, U> {
        #[pin]
        inner: FramedRead2<FramedWrite2<Fuse<T, U>>>,
    }
}

impl<T, U> Deref for Framed<T, U> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T, U> DerefMut for Framed<T, U> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T, U> Framed<T, U> {
    /// Release the I/O and Codec
    pub fn release(self) -> (T, U) {
        let fuse = self.inner.release().release();
        (fuse.t, fuse.u)
    }

    /// Consumes the `Framed`, returning its underlying I/O stream.
    ///
    /// Note that care should be taken to not tamper with the underlying stream
    /// of data coming in as it may corrupt the stream of frames otherwise
    /// being worked with.
    pub fn into_inner(self) -> T {
        self.release().0
    }

    /// Returns a reference to the underlying codec wrapped by
    /// `Framed`.
    ///
    /// Note that care should be taken to not tamper with the underlying codec
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn codec(&self) -> &U {
        &self.inner.u
    }

    /// Returns a mutable reference to the underlying codec wrapped by
    /// `Framed`.
    ///
    /// Note that care should be taken to not tamper with the underlying codec
    /// as it may corrupt the stream of frames otherwise being worked with.
    pub fn codec_mut(&mut self) -> &mut U {
        &mut self.inner.u
    }

    /// Returns a reference to the read buffer.
    pub fn read_buffer(&self) -> &BytesMut {
        self.inner.buffer()
    }
}

cfg_async! {
    use futures_sink::Sink;
    use futures_util::io::{AsyncRead, AsyncWrite};
    use futures_util::stream::{Stream, TryStreamExt};
    use std::marker::Unpin;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    impl<T, U> Framed<T, U>
    where
        T: AsyncRead + AsyncWrite,
        U: Decoder + Encoder,
    {
        /// Creates a new `Framed` transport with the given codec.
        /// A codec is a type which implements `Decoder` and `Encoder`.
        pub fn new(inner: T, codec: U) -> Self {
            Self {
                inner: FramedRead2::new(FramedWrite2::new(Fuse::new(inner, codec))),
            }
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

        fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
            self.project().inner.poll_ready(cx)
        }
        fn start_send(self: Pin<&mut Self>, item: U::Item) -> Result<(), Self::Error> {
            self.project().inner.start_send(item)
        }
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
            self.project().inner.poll_flush(cx)
        }
        fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
            self.project().inner.poll_close(cx)
        }
    }
}

cfg_blocking! {
    impl<T, U> Framed<T, U>
    where
        T: std::io::Read + std::io::Write,
        U: Decoder + Encoder,
    {
        /// Creates a new blocking `Framed` transport with the given codec.
        /// A codec is a type which implements `Decoder` and `Encoder`.
        pub fn new_blocking(inner: T, codec: U) -> Self {
            Self {
                inner: FramedRead2::new(FramedWrite2::new(Fuse::new(inner, codec))),
            }
        }
    }

    impl<T, U> Iterator for Framed<T, U>
    where
        T: std::io::Read,
        U: Decoder,
    {
        type Item = Result<U::Item, U::Error>;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next()
        }
    }

    impl<T, U> crate::IterSink<U::Item> for Framed<T, U>
    where
        T: std::io::Write,
        U: Encoder,
    {
        type Error = U::Error;

        fn start_send(&mut self, item: U::Item) -> Result<(), Self::Error> {
            self.inner.start_send(item)
        }

        fn ready(&mut self) -> Result<(), Self::Error> {
            self.inner.ready()
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            self.inner.flush()
        }
    }
}
