use futures::io::Cursor;
use futures::{executor, TryStreamExt};
use futures_codec::{BytesCodec, Framed};

#[test]
fn decodes() {
    let mut buf = [0u8; 32];
    let expected = buf.clone();
    let cur = Cursor::new(&mut buf[..]);
    let mut framed = Framed::new(cur, BytesCodec {});

    let read = executor::block_on(framed.try_next()).unwrap().unwrap();
    assert_eq!(&read[..], &expected[..]);

    assert!(executor::block_on(framed.try_next()).unwrap().is_none());
}
