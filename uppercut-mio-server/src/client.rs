use std::error::Error;

use mio::{Poll, Events, Token, Interest};
use mio::net::TcpStream;

use parser_combinators::stream::ByteStream;
use std::net::SocketAddr;
use std::io::{Read, Write};
use std::time::Duration;

pub struct Client {
    poll: Poll,
    events: Events,
    counter: usize,
}

impl Client {
    fn new() -> Self {
        let poll = Poll::new().unwrap();
        let events = Events::with_capacity(1024);

        Client {
            poll,
            events,
            counter: 0,
        }
    }

    fn connect(&mut self, addr: SocketAddr) -> Result<Connection, Box<dyn Error>> {
        let mut socket = TcpStream::connect(addr)?;

        self.counter += 1;
        self.poll.registry().register(&mut socket, Token(self.counter), Interest::WRITABLE).unwrap();

        let connection = Connection {
            socket: Some(socket),
            is_open: true,
            keep_alive: true,
            recv_buf: ByteStream::with_capacity(1024),
            send_buf: ByteStream::with_capacity(1024),
            can_read: false,
            can_write: false,
            buffer: [0 as u8; 1024],
        };

        Ok(connection)
    }
}

struct Connection {
    socket: Option<TcpStream>,
    is_open: bool,
    keep_alive: bool,
    recv_buf: ByteStream,
    send_buf: ByteStream,
    can_read: bool,
    can_write: bool,
    buffer: [u8; 1024],
}

impl Connection {
    fn send(&mut self, payload: &[u8]) {
        self.socket.as_ref().unwrap().write_all(payload).unwrap();
    }

    fn recv(&mut self) -> usize {
        let mut buf = [0 as u8; 1024];
        match self.socket.as_ref().unwrap().read(&mut buf) {
            Ok(0) | Err(_) => {
                self.is_open = false;
                0
            }
            Ok(n) => {
                self.recv_buf.put(&mut buf);
                n
            },
        }
    }
}

pub fn run() {
    let mut client = Client::new();

    let mut connection = client.connect("127.0.0.1:9000".parse().unwrap()).unwrap();

    loop {
        client.poll.poll(&mut client.events, Some(Duration::from_secs(1))).unwrap();

        let n = client.events.iter().count();
        println!("events: {}", n);

        break;
    }

    connection.send(b"hello there!");

    let received = connection.recv();
    println!("{}", received);
}
