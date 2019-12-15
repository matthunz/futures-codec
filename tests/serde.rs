use futures::io::Cursor;
use futures::{executor, SinkExt, StreamExt};
use futures_codec::{Framed, SerdeCodec};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    name: String,
    age: u8,
}

impl Person {
    fn new(name: &str, age: u8) -> Self {
        Self {
            name: name.into(),
            age,
        }
    }
}

#[test]
fn serializes_serde_enabled_structures() {
    let cur = Cursor::new(vec![0; 4096]);
    let mut framed = Framed::new(cur, SerdeCodec::default());

    let send_msgs = async {
        framed.send(Person::new("John", 11)).await.unwrap();
        framed.send(Person::new("Paul", 12)).await.unwrap();
        framed.send(Person::new("Mike", 13)).await.unwrap();
    };
    executor::block_on(send_msgs);

    let (mut cur, _) = framed.release();
    cur.set_position(0);
    let framed = Framed::new(cur, SerdeCodec::default());

    let recv_msgs = framed.take(3)
        .map(|res| res.unwrap())
        .collect::<Vec<_>>();
    let items: Vec<Person> = executor::block_on(recv_msgs);

    assert!(items == vec![
        Person::new("John", 11),
        Person::new("Paul", 12),
        Person::new("Mike", 13),
    ])
}
