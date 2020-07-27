use std::error::Error;
use std::net::SocketAddr;
use std::io::{Read, Write};
use std::time::Duration;

use mio::{Poll, Events, Token, Interest};
use mio::net::TcpStream;

use parser_combinators::stream::ByteStream;

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
    fn send(&mut self) {
        match self.socket.as_ref().unwrap().write_all(self.send_buf.as_ref()) {
            Ok(_) => {
                self.send_buf.clear();
            },
            Err(_) => {
                self.is_open = false;
            }
        }
    }

    fn recv(&mut self) -> usize {
        let mut buf = [0 as u8; 1024];
        match self.socket.as_ref().unwrap().read(&mut buf) {
            Ok(0) | Err(_) => {
                self.is_open = false;
                0
            }
            Ok(n) => {
                self.recv_buf.put(&buf[0..n]);
                n
            },
        }
    }
}

pub fn run() {
    let mut client = Client::new();

    let mut connection = client.connect("127.0.0.1:9000".parse().unwrap()).unwrap();

    connection.send_buf.put(b"hello there!");

    'outer: loop {
        client.poll.poll(&mut client.events, None).unwrap();

        for event in &client.events {
            println!("token={}: r={} w={} rc={} wc={}",
                     event.token().0,
                     event.is_readable(),
                     event.is_writable(),
                     event.is_read_closed(),
                     event.is_write_closed());

            if event.is_readable() {
                let _ = connection.recv();
                println!("rcvd: '{}'", String::from_utf8_lossy(connection.recv_buf.as_ref()));
                connection.is_open = !event.is_read_closed();
            }

            if event.is_writable() {
                connection.send();
                connection.send_buf.pull();
                connection.is_open = !event.is_write_closed();
            }

            if connection.is_open {
                client.poll.registry()
                    .reregister(connection.socket.as_mut().unwrap(),
                                Token(0),
                                Interest::READABLE.add(Interest::READABLE)).unwrap();
            } else {
                println!("connection is closed");
                break 'outer;
            }
        }
    }

    let _ = connection.socket.take();
    println!("done");
}
