use std::error::Error;
use std::net::SocketAddr;
use std::io::{Read, Write, ErrorKind};
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

        Ok(Connection::connected(socket, 32 * 1024))
    }
}

struct Connection {
    socket: Option<TcpStream>,
    is_open: bool,
    recv_buf: ByteStream,
    send_buf: ByteStream,
}

impl Default for Connection {
    fn default() -> Self {
        Self::empty(1024)
    }
}

impl Connection {
    fn empty(buffer_size: usize) -> Self {
        Self {
            socket: None,
            is_open: false,
            recv_buf: ByteStream::with_capacity(buffer_size),
            send_buf: ByteStream::with_capacity(buffer_size),
        }
    }

    fn connected(socket: TcpStream, buffer_size: usize) -> Self {
        Self {
            socket: Some(socket),
            is_open: true,
            recv_buf: ByteStream::with_capacity(buffer_size),
            send_buf: ByteStream::with_capacity(buffer_size),
        }
    }
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
        let mut buf = [0u8; 1024];
        let mut bytes_received: usize = 0;
        loop {
            match self.socket.as_ref().unwrap().read(&mut buf) {
                Ok(n) if n > 0 => {
                    self.recv_buf.put(&buf[0..n]);
                    bytes_received += n;
                },
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Ok(_) | Err(_) => {
                    self.is_open = false;
                    break
                }
            }
        }
        bytes_received
    }
}

pub fn run() {
    let mut client = Client::new();

    let mut connection = client.connect("127.0.0.1:9000".parse().unwrap()).unwrap();

    connection.send_buf.put(b"hello there!");

    'outer: loop {
        client.poll.poll(&mut client.events, None).unwrap();

        for event in &client.events {
            if event.is_readable() {
                let _ = connection.recv();
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
