extern crate log;
extern crate env_logger;
use log::debug;

use std::error::Error;
use std::net::SocketAddr;
use std::io::{Read, Write, ErrorKind};
use std::time::Duration;

use mio::{Poll, Events, Token, Interest};
use mio::net::TcpStream;

use parser_combinators::stream::ByteStream;
use std::collections::HashMap;

// TODO make poll timeout, buffer sizes, pooling, etc configurable by introducing ClientConfig

pub struct Client {
    poll: Poll,
    events: Events,
    counter: usize,
    connections: HashMap<usize, Connection>,
    destinations: HashMap<String, usize>,
}

impl Client {
    fn new() -> Self {
        let poll = Poll::new().unwrap();
        let events = Events::with_capacity(1024);

        Client {
            poll,
            events,
            counter: 0,
            connections: HashMap::new(),
            destinations: HashMap::new(),
        }
    }

    fn connect(&mut self, addr: SocketAddr) -> Result<usize, Box<dyn Error>> {
        self.counter += 1;

        let mut socket = TcpStream::connect(addr)?;
        let id = self.counter;
        self.poll.registry().register(&mut socket, Token(id), Interest::WRITABLE).unwrap();

        let connection = Connection::connected(id, addr.to_string(), socket, 1024);
        self.connections.insert(id, connection);
        self.destinations.insert(addr.to_string(), id);
        Ok(id)
    }

    fn poll(&mut self, timeout: Option<Duration>) {
        self.poll.poll(&mut self.events, timeout).unwrap();

        for event in &self.events {
            let id = event.token().0;

            let mut connection = self.connections.remove(&id).unwrap();

            if event.is_readable() {
                debug!("connection {} is readable.", id);
                let _ = connection.recv();
                connection.is_open = !event.is_read_closed();
            }

            if event.is_writable() {
                debug!("connection {} is writable.", id);
                connection.send();
                connection.send_buf.pull();
                connection.is_open = !event.is_write_closed();
            }

            if connection.is_open {
                self.poll.registry()
                    .reregister(connection.socket.as_mut().unwrap(),
                                event.token(),
                                Interest::READABLE.add(Interest::READABLE)).unwrap();

                self.connections.insert(id, connection);
            } else {
                self.destinations.remove(&connection.target);
                debug!("connection {} is closed.", id);
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }

    fn has(&self, addr: &str) -> bool {
        self.destinations.contains_key(addr)
    }

    fn put(&mut self, addr: &str, payload: &[u8]) -> usize {
        let target = addr.to_string();
        let id = self.destinations.get(&target).unwrap();
        self.connections.get_mut(id).unwrap().send_buf.put(payload)
    }

    fn get(&mut self, addr: &str) -> Option<&mut ByteStream> {
        let target = addr.to_string();
        let id = self.destinations.get(&target).unwrap();
        self.connections.get_mut(id).map(|c| &mut c.recv_buf)
    }
}

struct Connection {
    token: Token,
    target: String,
    socket: Option<TcpStream>,
    is_open: bool,
    recv_buf: ByteStream,
    send_buf: ByteStream,
    buffer: [u8; 1024],
}

impl Connection {
    fn connected(id: usize, target: String, socket: TcpStream, buffer_size: usize) -> Self {
        Self {
            token: Token(id),
            target,
            socket: Some(socket),
            is_open: true,
            recv_buf: ByteStream::with_capacity(buffer_size),
            send_buf: ByteStream::with_capacity(buffer_size),
            buffer: [0u8; 1024],
        }
    }
}

impl Connection {
    fn send(&mut self) {
        match self.socket.as_ref().unwrap().write_all(self.send_buf.as_ref()) {
            Ok(_) => {
                debug!("connection {} sent {} bytes.", self.token.0, self.send_buf.len());
                self.send_buf.clear();
            },
            Err(_) => {
                self.is_open = false;
                debug!("connection {} was closed on send.", self.token.0);
            }
        }
    }

    fn recv(&mut self) -> usize {
        let mut bytes_received: usize = 0;
        while self.recv_buf.cap() >= self.buffer.len() {
            match self.socket.as_ref().unwrap().read(&mut self.buffer) {
                Ok(n) if n > 0 => {
                    debug!("connection {} received {} bytes.", self.token.0, n);
                    self.recv_buf.put(&self.buffer[0..n]);
                    bytes_received += n;
                },
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Ok(_) | Err(_) => {
                    debug!("connection {} was closed on recv.", self.token.0);
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

    let target = "127.0.0.1:9000"; // nc -l 9000
    // let target = "216.58.215.78:80"; // ping google.com

    let _id = client.connect(target.parse().unwrap()).unwrap();

    client.put(target, b"hello there!");
    // client.put(target, b"GET / HTTP/1.1\r\n\r\n");

    while !client.is_empty() {
        client.poll(Some(Duration::from_secs(1)));

        if client.has(target) {
            let stream = client.get(target).unwrap();
            if stream.len() > 0 {
                let len = stream.len();
                let bytes = stream.get(len).unwrap();
                stream.clear();
                debug!("'{}'", String::from_utf8_lossy(&bytes));
            }
        }
    }
}
