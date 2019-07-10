# futures_codec

Utilities for encoding and decoding frames using async/await.

Contains adapters to go from streams of bytes, `AsyncRead` and `AsyncWrite`,
to framed streams implementing `Sink` and `Stream`. Framed streams are also known as transports.

[Docs](https://docs.rs/futures_codec) | [Crate](https://crates.io/crates/futures_codec)

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
