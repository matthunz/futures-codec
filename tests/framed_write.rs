use futures::executor;
use futures::sink::SinkExt;
use futures_codec::{FramedWrite, LinesCodec};
use std::io::Cursor;

#[test]
fn line_write() {
    let curs = Cursor::new(vec![0u8; 16]);
    let mut framer = FramedWrite::new(curs, LinesCodec {});
    executor::block_on(framer.send("Hello\n".to_owned())).unwrap();
    executor::block_on(framer.send("World\n".to_owned())).unwrap();
    let (curs, _) = framer.release();
    assert_eq!(&curs.get_ref()[0..12], b"Hello\nWorld\n");
    assert_eq!(curs.position(), 12);
}

#[test]
fn line_write_to_eof() {
    let curs = Cursor::new(vec![0u8; 16]);
    let mut framer = FramedWrite::new(curs, LinesCodec {});
    let _err =
        executor::block_on(framer.send("This will fill up the buffer\n".to_owned())).unwrap_err();
    let (curs, _) = framer.release();
    assert_eq!(curs.position(), 16);
    assert_eq!(&curs.get_ref()[0..16], b"This will fill u");
}
