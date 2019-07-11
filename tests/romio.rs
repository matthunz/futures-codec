#![feature(async_await)]

use
{
	romio::tcp    :: { TcpListener, TcpStream } ,
	futures       :: { prelude::*             } ,
	async_runtime :: { rt, RtConfig           } ,
	futures_codec :: { Framed, LinesCodec     } ,
};


// This test caught a bug in FramedWrite2::poll_flush. The written out bytes did not get removed
// from the buffer. This test assures correct integration of the LinesCodec, both encoding and
// decoding.
//
#[test]
//
fn receive_twice()
{
	rt::init( RtConfig::Local ).expect( "rt::init" );

	let server = async
	{
		let     socket_addr  = "127.0.0.1:3323".parse().expect( "parse address" );
		let mut listener     = TcpListener::bind(&socket_addr).expect( "bind tcp" );
		let mut incoming     = listener.incoming();
		let stream           = incoming.next().await.expect( "get stream" ).expect( "get stream" );

		let mut framed = Framed::new( stream, LinesCodec {} );

		framed.send( "A line\n"       .to_string() ).await.expect( "Send a line"        );
		framed.send( "A second line\n".to_string() ).await.expect( "Send a second line" );
		framed.send( "A third line\n" .to_string() ).await.expect( "Send a third line"  );
	};

	let client = async
	{
		let     socket_addr  = "127.0.0.1:3323".parse().expect( "parse address" );
		let stream = TcpStream::connect(&socket_addr).await.expect( "connect tcp" );

		let mut framed = Framed::new( stream, LinesCodec {} );


		let res = framed.next().await.expect( "Receive some" ).expect( "Receive a line" );
		dbg!( &res );
		assert_eq!( "A line\n".to_string(), res );


		let res = framed.next().await.expect( "Receive some" ).expect( "Receive a second line" );
		dbg!( &res );
		assert_eq!( "A second line\n".to_string(), res );


		let res = framed.next().await.expect( "Receive some" ).expect( "Receive a second line" );
		dbg!( &res );
		assert_eq!( "A third line\n".to_string(), res );


		let res = framed.next().await;
		dbg!( &res );
		assert!( res.is_none() );
	};

	rt::spawn( server ).expect( "spawn task" );
	rt::spawn( client ).expect( "spawn task" );

	rt::run();
}
