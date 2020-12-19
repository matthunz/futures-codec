use futures::executor;
use futures::stream::StreamExt;
use futures::AsyncRead;
use futures_codec::{BytesCodec, BytesMut, Decoder, FramedRead, LinesCodec};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

// Sends two lines at once, then nothing else forever
struct MockBurstySender {
    sent: bool,
}

impl AsyncRead for MockBurstySender {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        const MESSAGES: &[u8] = b"one\ntwo\n";
        if !self.sent && buf.len() >= MESSAGES.len() {
            self.sent = true;
            buf[0..MESSAGES.len()].clone_from_slice(MESSAGES);
            Poll::Ready(Ok(MESSAGES.len()))
        } else {
            Poll::Pending
        }
    }
}

impl io::Read for MockBurstySender {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        const MESSAGES: &[u8] = b"one\ntwo\n";
        if !self.sent && buf.len() >= MESSAGES.len() {
            self.sent = true;
            buf[0..MESSAGES.len()].clone_from_slice(MESSAGES);
            Ok(MESSAGES.len())
        } else {
            Ok(0)
        }
    }
}

#[test]
fn line_read_multi() {
    let io = MockBurstySender { sent: false };
    let mut framed = FramedRead::new(io, LinesCodec {});
    let one = executor::block_on(StreamExt::next(&mut framed))
        .unwrap()
        .unwrap();
    assert_eq!(one, "one\n");
    let two = executor::block_on(StreamExt::next(&mut framed))
        .unwrap()
        .unwrap();
    assert_eq!(two, "two\n");
}

#[test]
fn blocking_line_read_multi() {
    let io = MockBurstySender { sent: false };
    let mut framed = FramedRead::new_blocking(io, LinesCodec {});
    let one = Iterator::next(&mut framed).unwrap().unwrap();
    assert_eq!(one, "one\n");
    let two = Iterator::next(&mut framed).unwrap().unwrap();
    assert_eq!(two, "two\n");
}

struct OneByteAtATime<'a> {
    input: &'a [u8],
}

impl AsyncRead for OneByteAtATime<'_> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if self.input.is_empty() {
            Poll::Ready(Ok(0))
        } else {
            buf[0] = self.input[0];
            self.input = &self.input[1..];
            Poll::Ready(Ok(1))
        }
    }
}

impl io::Read for OneByteAtATime<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.input.is_empty() {
            Ok(0)
        } else {
            buf[0] = self.input[0];
            self.input = &self.input[1..];
            Ok(1)
        }
    }
}

/// A decoder that only returns `a` characters from the input.
struct AllTheAs;

impl Decoder for AllTheAs {
    type Item = char;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        while !src.is_empty() {
            let buf = src.split_to(1);
            let c = char::from(buf[0]);
            if c == 'a' {
                return Ok(Some(c));
            }
        }
        Ok(None)
    }
}

#[test]
fn read_few_messages() {
    let string: &[u8] = b"aabbbabbbabbbabb";
    let input = OneByteAtATime { input: string };
    let mut framed = FramedRead::new(input, AllTheAs);
    for _ in 0..5 {
        let item = executor::block_on(StreamExt::next(&mut framed))
            .unwrap()
            .unwrap();
        assert_eq!(item, 'a');
    }
}

#[test]
fn blocking_read_few_messages() {
    let string: &[u8] = b"aabbbabbbabbbabb";
    let input = OneByteAtATime { input: string };
    let mut framed = FramedRead::new_blocking(input, AllTheAs);
    for _ in 0..5 {
        let item = Iterator::next(&mut framed).unwrap().unwrap();
        assert_eq!(item, 'a');
    }
}

#[test]
fn blocking_decode_bytes() {
    let mut buf = [0u8; 32];
    let expected = buf;
    let cur = std::io::Cursor::new(&mut buf[..]);
    let mut framed = FramedRead::new_blocking(cur, BytesCodec {});

    let read = framed.next().unwrap().unwrap();
    assert_eq!(&read[..], &expected[..]);

    assert!(framed.next().is_none());
}

#[test]
fn blocking_decode_lines() {
    let buf = "Hello\nWorld\nError\n".to_owned();
    let cur = std::io::Cursor::new(&buf[..buf.len() - 1]);

    let mut framed = FramedRead::new_blocking(cur, LinesCodec {});

    let next = framed.next().unwrap().unwrap();
    assert_eq!(next, "Hello\n");

    let next = framed.next().unwrap().unwrap();
    assert_eq!(next, "World\n");

    let next = framed.next();
    assert!(next.is_none());

    let cur = std::io::Cursor::new(buf);

    let mut framed = FramedRead::new_blocking(cur, LinesCodec {});

    let next = framed.next().unwrap().unwrap();
    assert_eq!(next, "Hello\n");

    let next = framed.next().unwrap().unwrap();
    assert_eq!(next, "World\n");

    let next = framed.next().unwrap().unwrap();
    assert_eq!(next, "Error\n");
}
