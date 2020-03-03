use std::io::Error as IoError;
use std::marker::PhantomData;

use crate::{Decoder, Encoder};
use bytes::{Buf, BufMut, BytesMut};

use serde::{Deserialize, Serialize};
use serde_cbor::Error as CborError;

/// A codec for JSON encoding and decoding using serde_cbor
/// Enc is the type to encode, Dec is the type to decode
/// ```
/// # use futures::{executor, SinkExt, TryStreamExt};
/// # use futures::io::Cursor;
/// use serde::{Serialize, Deserialize};
/// use futures_codec::{CborCodec, Framed};
///
/// #[derive(Serialize, Deserialize)]
/// struct Something {
///     pub data: u16,
/// }
///
/// async move {
///     # let mut buf = vec![];
///     # let stream = Cursor::new(&mut buf);
///     // let stream = ...
///     let codec = CborCodec::<Something, Something>::new();
///     let mut framed = Framed::new(stream, codec);
///
///     while let Some(s) = framed.try_next().await.unwrap() {
///         println!("{:?}", s.data);
///     }
/// };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CborCodec<Enc, Dec> {
    enc: PhantomData<Enc>,
    dec: PhantomData<Dec>,
    packed: bool,
}

/// JSON Codec error enumeration
#[derive(Debug)]
pub enum CborCodecError {
    /// IO error
    Io(IoError),
    /// JSON error
    Cbor(CborError),
}

impl From<IoError> for CborCodecError {
    fn from(e: IoError) -> CborCodecError {
        return CborCodecError::Io(e);
    }
}

impl From<CborError> for CborCodecError {
    fn from(e: CborError) -> CborCodecError {
        return CborCodecError::Cbor(e);
    }
}

impl<Enc, Dec> CborCodec<Enc, Dec>
where
    for<'de> Dec: Deserialize<'de> + 'static,
    for<'de> Enc: Serialize + 'static,
{
    /// Creates a new `CborCodec` with the associated types
    pub fn new() -> CborCodec<Enc, Dec> {
        CborCodec {
            enc: PhantomData,
            dec: PhantomData,
            packed: false,
        }
    }

    /// When `true`, encode CBOR values in packed format.
    ///
    /// In the packed format enum variant and struct field names are replaced
    /// with numeric indices to conserve space. This may, however, reduce
    /// portability. See [section 3.7] of the CBOR specification.
    ///
    /// The default value is `false`, i.e. variant and field names are encoded
    /// as UTF-8 strings.
    ///
    /// [section 3.7]: https://tools.ietf.org/html/rfc7049#section-3.7
    pub fn set_packed(mut self, packed: bool) -> Self {
        self.packed = packed;
        self
    }
}

/// Decoder impl parses cbor objects from bytes
impl<Enc, Dec> Decoder for CborCodec<Enc, Dec>
where
    for<'de> Dec: Deserialize<'de> + 'static,
    for<'de> Enc: Serialize + 'static,
{
    type Item = Dec;
    type Error = CborCodecError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Build deserializer
        let mut de = serde_cbor::Deserializer::from_slice(&buf);

        // Attempt deserialization
        let res: Result<Dec, _> = serde::de::Deserialize::deserialize(&mut de);

        // If we ran out before parsing, return none and try again later
        let res = match res {
            Ok(v) => Ok(Some(v)),
            Err(e) if e.is_eof() => Ok(None),
            Err(e) => Err(e.into()),
        };

        // Update offset from iterator
        let offset = de.byte_offset();

        // Advance buffer
        buf.advance(offset);

        res
    }
}

/// Encoder impl encodes object streams to bytes
impl<Enc, Dec> Encoder for CborCodec<Enc, Dec>
where
    for<'de> Dec: Deserialize<'de> + 'static,
    for<'de> Enc: Serialize + 'static,
{
    type Item = Enc;
    type Error = CborCodecError;

    fn encode(&mut self, data: Self::Item, buf: &mut BytesMut) -> Result<(), Self::Error> {
        // Encode cbor
        let j = if self.packed {
            serde_cbor::ser::to_vec_packed(&data)?
        } else {
            serde_cbor::to_vec(&data)?
        };

        // Write to buffer
        buf.reserve(j.len());
        buf.put_slice(&j);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;
    use serde::{Deserialize, Serialize};

    use super::CborCodec;
    use crate::{Decoder, Encoder};

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        pub name: String,
        pub data: u16,
    }

    #[test]
    fn cbor_codec_encode_decode() {
        let mut codec = CborCodec::<TestStruct, TestStruct>::new();
        let mut buff = BytesMut::new();

        let item1 = TestStruct {
            name: "Test name".to_owned(),
            data: 16,
        };
        codec.encode(item1.clone(), &mut buff).unwrap();

        let item2 = codec.decode(&mut buff).unwrap().unwrap();
        assert_eq!(item1, item2);

        assert_eq!(codec.decode(&mut buff).unwrap(), None);

        assert_eq!(buff.len(), 0);
    }

    #[test]
    fn cbor_codec_partial_decode() {
        let mut codec = CborCodec::<TestStruct, TestStruct>::new();
        let mut buff = BytesMut::new();

        let item1 = TestStruct {
            name: "Test name".to_owned(),
            data: 34,
        };
        codec.encode(item1.clone(), &mut buff).unwrap();

        let mut start = buff.clone().split_to(4);
        assert_eq!(codec.decode(&mut start).unwrap(), None);

        codec.decode(&mut buff).unwrap().unwrap();

        assert_eq!(buff.len(), 0);
    }

    #[test]
    fn cbor_codec_packed_encode_decode() {
        let mut codec = CborCodec::<TestStruct, TestStruct>::new().set_packed(true);
        let mut buff = BytesMut::new();

        let item1 = TestStruct {
            name: "Test name".to_owned(),
            data: 42,
        };
        codec.encode(item1.clone(), &mut buff).unwrap();

        let item2 = codec.decode(&mut buff).unwrap().unwrap();
        assert_eq!(item1, item2);
    }
}
