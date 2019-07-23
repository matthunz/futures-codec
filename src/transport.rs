use crate::{Decoder, Encoder};
use bytes::BytesMut;
use futures::io::{AsyncRead, AsyncWrite};
use std::marker::Unpin;

pub type FramedRead<T, U> = Transport<T, U, FramedPart, Empty>;

impl<T, U> FramedRead<T, U>
where
    T: AsyncRead + Unpin,
    U: Decoder,
{
    pub fn new(io: T, decoder: U) -> Self {
        Transport::framed_read(io, decoder)
    }
}

pub struct Empty {
    _priv: (),
}

impl Empty {
    fn new() -> Self {
        Self { _priv: () }
    }
}

pub struct FramedPart {
    buf: BytesMut,
}

pub struct Transport<T, U, A, B> {
    io: T,
    codec: U,
    parts: (A, B),
}

impl<T, U> Transport<T, U, FramedPart, FramedPart>
where
    T: AsyncRead + AsyncWrite + Unpin,
    U: Decoder + Encoder,
{
    pub fn framed(io: T, codec: U) -> Self {
        Transport {
            io,
            codec,
            parts: (
                FramedPart {
                    buf: BytesMut::new(),
                },
                FramedPart {
                    buf: BytesMut::new(),
                },
            ),
        }
    }
}

impl<T, U> Transport<T, U, FramedPart, Empty>
where
    T: AsyncRead + Unpin,
    U: Decoder,
{
    pub fn framed_read(io: T, decoder: U) -> Self {
        Transport {
            io,
            codec: decoder,
            parts: (
                FramedPart {
                    buf: BytesMut::new(),
                },
                Empty::new(),
            ),
        }
    }
}

impl<T, U> Transport<T, U, Empty, FramedPart>
where
    T: AsyncWrite + Unpin,
    U: Encoder,
{
    pub fn framed_write(io: T, encoder: U) -> Transport<T, U, Empty, FramedPart> {
        Transport {
            io,
            codec: encoder,
            parts: (
                Empty::new(),
                FramedPart {
                    buf: BytesMut::new(),
                },
            ),
        }
    }
}

impl<T, U, B> Transport<T, U, FramedPart, B> {
    /// Stream
    pub fn next(&self) {}
}

impl<T, U, A> Transport<T, U, A, FramedPart> {
    /// Sink
    pub fn send(&self) {}
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::LinesCodec;
    use std::io::Cursor;

    #[test]
    fn it_works() {
        let io = Cursor::new(vec![0; 3]);
        let tp = Transport::framed(io, LinesCodec {});
        tp.next();
        tp.send();
    }
}
