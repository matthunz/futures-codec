use super::Decoder;
use super::framed::Fuse;
use bytes::BytesMut;
use futures::{ready, TryStream};
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