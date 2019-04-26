use crate::{Decoder, Encoder};

pub struct BytesCodec {}

impl Decoder for BytesCodec {}

impl Encoder for BytesCodec {}

#[cfg(test)]
mod tests {
    use super::BytesCodec;
    use crate::{FramedRead, FramedWrite};

    #[test]
    fn it_reads() {
        let buf = b"Hello World!";
        let _framed = FramedRead::new(&buf[..], BytesCodec {});
    }

    #[test]
    fn it_writes() {
        let mut buf = Vec::new();
        let _framed = FramedWrite::new(&mut buf, BytesCodec {});
    }
}