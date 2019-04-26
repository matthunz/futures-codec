use super::Decoder;
use super::framed::Fuse;
use bytes::BytesMut;
use futures::{ready, TryStream};
use futures::io::AsyncRead;
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
///     let msg = await!(framed.try_next()).unwrap().unwrap();
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
    pub fn new(inner: T, decoder: D) -> Self {
        Self {
            inner: framed_read_2(Fuse(inner, decoder)),
        }
    }
}

impl<T, D> TryStream for FramedRead<T, D>
where
    T: AsyncRead + Unpin,
    D: Decoder
{
    type Ok = D::Item;
    type Error = D::Error;

    fn try_poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Ok, Self::Error>>> {
        Pin::new(&mut self.inner).try_poll_next(cx)
    }
} 

pub struct FramedRead2<T> {
    inner: T,
    buffer: BytesMut
}

const INITIAL_CAPACITY: usize = 8 * 1024;

pub fn framed_read_2<T>(inner: T) -> FramedRead2<T> {
    FramedRead2 { inner, buffer: BytesMut::with_capacity(INITIAL_CAPACITY) }
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
        let this = &mut *self;
        let mut buf = [0u8; INITIAL_CAPACITY];
        
        loop {
            let n = ready!(Pin::new(&mut this.inner).poll_read(cx, &mut buf))?;
            this.buffer.extend_from_slice(&buf[..n]);
            
            if let Some(item) = this.inner.decode(&mut this.buffer)? {
                return Poll::Ready(Some(Ok(item)));
            }
        }
    }
}