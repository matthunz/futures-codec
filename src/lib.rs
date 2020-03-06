#![deny(missing_docs)]
//! Utilities for encoding and decoding frames using `async/await`.
//!
//! Contains adapters to go from streams of bytes, [`AsyncRead`](futures::io::AsyncRead)
//! and [`AsyncWrite`](futures::io::AsyncWrite), to framed streams implementing [`Sink`](futures::Sink) and [`Stream`](futures::Stream).
//! Framed streams are also known as `transports`.
//!
//! ```
//! # futures::executor::block_on(async move {
//! use futures::TryStreamExt;
//! use futures::io::Cursor;
//! use futures_codec::{LinesCodec, Framed};
//!
//! let io = Cursor::new(Vec::new());
//! let mut framed = Framed::new(io, LinesCodec);
//!
//! while let Some(line) = framed.try_next().await? {
//!     dbg!(line);
//! }
//! # Ok::<_, std::io::Error>(())
//! # }).unwrap();
//! ```

mod codec;
pub use codec::{BytesCodec, LengthCodec, LinesCodec};
pub use bytes::{Bytes, BytesMut};

#[cfg(feature = "cbor")]
pub use codec::{CborCodec, CborCodecError};
#[cfg(feature = "json")]
pub use codec::{JsonCodec, JsonCodecError};

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

mod fuse;
