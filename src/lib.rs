mod codec;
pub use codec::BytesCodec;

mod framed;
pub use framed::Framed;

mod framed_read;
pub use framed_read::FramedRead;

mod framed_write;
pub use framed_write::FramedWrite;

pub trait Decoder {}

pub trait Encoder {}