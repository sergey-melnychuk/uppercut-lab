extern crate env_logger;
extern crate log;
use log::debug;

use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::io::{Read, Write};

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use uppercut::api::{AnyActor, AnySender, Envelope};

use crate::protocol::process;
use parsed::stream::ByteStream;

#[derive(Debug)]
pub struct Start;

#[derive(Debug)]
struct Loop;

struct Connect {
    socket: Option<TcpStream>,
    keep_alive: bool,
    handler: fn(&mut ByteStream, &mut ByteStream),
}

impl Debug for Connect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connect")
            .field("socket", &self.socket)
            .field("keep_alive", &self.keep_alive)
            .field("handler", &"<function>")
            .finish()
    }
}

#[derive(Debug)]
struct Work {
    is_readable: bool,
    is_writable: bool,
}

pub struct Listener {
    poll: Poll,
    events: Events,
    socket: TcpListener,
    counter: usize,
}

impl Listener {
    pub fn listen(addr: &str) -> Result<Listener, Box<dyn Error>> {
        let poll = Poll::new().unwrap();
        let events = Events::with_capacity(1024);
        let addr = addr.parse()?;
        let mut socket = TcpListener::bind(addr)?;
        poll.registry()
            .register(&mut socket, Token(0), Interest::READABLE)
            .unwrap();

        let listener = Listener {
            poll,
            events,
            socket,
            counter: 0,
        };
        Ok(listener)
    }
}

impl AnyActor for Listener {
    fn receive(&mut self, envelope: Envelope, sender: &mut dyn AnySender) {
        if envelope.message.downcast_ref::<Loop>().is_some() {
            self.poll.poll(&mut self.events, None).unwrap();
            for event in self.events.iter() {
                match event.token() {
                    Token(0) => {
                        while let Ok((mut socket, _)) = self.socket.accept() {
                            debug!("connected: {}", self.counter + 1);
                            self.counter += 1;
                            let token = Token(self.counter);
                            self.poll
                                .registry()
                                .register(
                                    &mut socket,
                                    token,
                                    Interest::READABLE | Interest::WRITABLE,
                                )
                                .unwrap();
                            let tag = format!("{}", self.counter);
                            sender.spawn(&tag, Box::new(|| Box::new(Connection::default())));
                            let connect = Connect {
                                socket: Some(socket),
                                keep_alive: true,
                                handler: process,
                            };
                            sender.send(&tag, Envelope::of(connect));
                        }
                    }
                    token => {
                        let tag = format!("{}", token.0);
                        let work = Work {
                            is_readable: event.is_readable(),
                            is_writable: event.is_writable(),
                        };
                        sender.send(&tag, Envelope::of(work));
                    }
                }
            }
            sender.send(sender.me(), Envelope::of(Loop));
        } else if envelope.message.downcast_ref::<Start>().is_some() {
            sender.send(sender.me(), Envelope::of(Loop));
        }
    }
}

struct Connection {
    socket: Option<TcpStream>,
    is_open: bool,
    keep_alive: bool,
    recv_buf: ByteStream,
    send_buf: ByteStream,
    can_recv: bool,
    can_send: bool,
    handler: Option<fn(&mut ByteStream, &mut ByteStream)>,
}

impl Connection {
    fn keep_open(&mut self, sender: &mut dyn AnySender) -> bool {
        if !self.is_open {
            if !self.keep_alive {
                self.is_open = true;
            } else {
                self.socket = None;
                sender.stop(sender.me());
            }
        }
        self.is_open
    }
}

impl Default for Connection {
    fn default() -> Self {
        Connection {
            socket: None,
            is_open: true,
            keep_alive: true,
            recv_buf: ByteStream::with_capacity(1024),
            send_buf: ByteStream::with_capacity(1024),
            can_recv: false,
            can_send: false,
            handler: None,
        }
    }
}

impl AnyActor for Connection {
    fn receive(&mut self, mut envelope: Envelope, sender: &mut dyn AnySender) {
        if let Some(connect) = envelope.message.downcast_mut::<Connect>() {
            self.socket = connect.socket.take();
            self.keep_alive = connect.keep_alive;
            self.handler = Some(connect.handler);
        } else if self.socket.is_none() {
            sender.send(sender.me(), envelope);
        } else if let Some(work) = envelope.message.downcast_ref::<Work>() {
            let mut buffer = [0u8; 1024];
            debug!("work: {:?}", work);
            self.can_recv = work.is_readable;
            self.can_send = self.can_send || work.is_writable;
            if self.can_recv {
                debug!("connection {} is readable", sender.me());
                match self.socket.as_ref().unwrap().read(&mut buffer[..]) {
                    Ok(0) | Err(_) => {
                        debug!("connection {} closed (read 0 bytes)", sender.me());
                        self.is_open = false;
                    }
                    Ok(n) => {
                        debug!("connection {} read {} bytes", sender.me(), n);
                        self.recv_buf.put(&buffer[0..n]);
                    }
                }
            }

            if !self.keep_open(sender) {
                return;
            }

            if self.recv_buf.len() > 0 {
                self.handler.as_ref().unwrap()(&mut self.recv_buf, &mut self.send_buf);
            }

            if self.can_send && self.send_buf.len() > 0 {
                debug!("connection {} is writable", sender.me());
                match self
                    .socket
                    .as_ref()
                    .unwrap()
                    .write_all(self.send_buf.as_ref())
                {
                    Ok(_) => {
                        debug!(
                            "connection {} written {} bytes",
                            sender.me(),
                            self.send_buf.len()
                        );
                        self.send_buf.clear();
                    }
                    _ => {
                        debug!("connection {} closed (write failed)", sender.me());
                        self.is_open = false;
                    }
                }
            }

            self.keep_open(sender);
        }
    }
}
