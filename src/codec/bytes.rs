use crate::Decoder;

pub struct BytesCodec {}

impl Decoder for BytesCodec {}

#[cfg(test)]
mod tests {
    use super::BytesCodec;
    use crate::FramedRead;

    #[test]
    fn it_reads() {
        let buf = b"Hello World!";
        let _framed = FramedRead::new(&buf[..], BytesCodec {});
    }
}