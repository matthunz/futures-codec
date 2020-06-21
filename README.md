# futures_codec

Utilities for encoding and decoding frames using async/await.

Contains adapters to go from streams of bytes, `AsyncRead` and `AsyncWrite`,
to framed streams implementing `Sink` and `Stream`. Framed streams are also known as transports.

[![Latest Version](https://img.shields.io/crates/v/futures-codec.svg)](https://crates.io/crates/futures-codec)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/futures-codec)
[![Build Status](https://travis-ci.com/matthunz/futures-codec.svg)](https://travis-ci.com/matthunz/futures-codec)
![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)


### Example
```rust
use futures_codec::{LinesCodec, Framed};

async fn main() {
    // let stream = ...
    let mut framed = Framed::new(stream, LinesCodec {});

    while let Some(line) = framed.try_next().await.unwrap() {
        println!("{:?}", line);
    }
}
```
