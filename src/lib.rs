//! Utilities for encoding and decoding frames using `async/await`.
//!
//! Contains adapters to go from streams of bytes, [`AsyncRead`](futures::io::AsyncRead)
//! and [`AsyncWrite`](futures::io::AsyncWrite), to framed streams implementing [`Sink`](futures::Sink) and [`Stream`](futures::Stream).
//! Framed streams are also known as `transports`.
//!
//! ```
//! # #![feature(async_await, await_macro)]
//! # use futures::{executor, SinkExt, TryStreamExt};
//! # use std::io::Cursor;
//! use futures_codec::{LinesCodec, Framed};
//!
//! async move {
//!     # let mut buf = vec![];
//!     # let stream = Cursor::new(&mut buf);
//!     // let stream = ...
//!     let mut framed = Framed::new(stream, LinesCodec {});
//!
//!     while let Some(line) = framed.try_next().await.unwrap() {
//!         println!("{:?}", line);
//!     }
//! };
//! ```

mod codec;
pub use codec::{BytesCodec, LinesCodec};

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
