//! Utilities for encoding and decoding frames using `async/await`.
//!
//! Contains adapters to go from streams of bytes, [`AsyncRead`](futures::io::AsyncRead)
//! and [`AsyncWrite`](futures::io::AsyncWrite), to framed streams implementing [`Sink`](futures::Sink) and [`Stream`](futures::Stream).
//! Framed streams are also known as `transports`.
//!
//! ```
//! # #![feature(async_await, await_macro)]
//! # use futures::{SinkExt, TryStreamExt};
//! # use std::io::Cursor;
//! use bytes::Bytes;
//! use futures_codec::{BytesCodec, Framed};
//!
//! async move {
//!     # let mut buf = vec![];
//!     # let stream = Cursor::new(&mut buf);
//!     let mut framed = Framed::new(stream, BytesCodec {});
//! 
//!     let msg = Bytes::from("Hello World!");
//!     await!(framed.send(msg)).unwrap();
//!
//!     while let Some(msg) = await!(framed.try_next()).unwrap() {
//!         println!("{:?}", msg);
//!     }
//! };
//! ```

mod codec;
pub use codec::BytesCodec;

mod decoder;
pub use decoder::Decoder;

mod encoder;
pub use encoder::Encoder;

mod framed;
pub use framed::Framed;

mod framed_read;
pub use framed_read::FramedRead;

mod framed_write;
pub use framed_write::FramedWrite;