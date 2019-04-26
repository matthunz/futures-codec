mod codec;
pub use codec::BytesCodec;

mod framed_read;
pub use framed_read::FramedRead;

struct Fuse<T, U>(T, U);

pub trait Decoder {}