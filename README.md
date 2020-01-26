# futures_codec

Utilities for encoding and decoding frames using async/await.

Contains adapters to go from streams of bytes, `AsyncRead` and `AsyncWrite`,
to framed streams implementing `Sink` and `Stream`. Framed streams are also known as transports.

![https://crates.io/crates/futures_codec](https://img.shields.io/crates/v/futures_codec.svg)
![https://travis-ci.com/matthunz/futures-codec](https://travis-ci.com/matthunz/futures-codec.svg)
![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg) |
[Docs](https://docs.rs/futures_codec)


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
