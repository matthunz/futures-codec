mod codec;
pub use codec::BytesCodec;

mod framed_read;
pub use framed_read::FramedRead;

mod framed_write;
pub use framed_write::FramedWrite;

struct Fuse<T, U>(T, U);

pub trait Decoder {}

pub trait Encoder {}