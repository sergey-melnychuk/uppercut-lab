use std::error::Error;
use std::io::{Read, Write};
use std::time::Duration;

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use uppercut::api::{AnyActor, Envelope, AnySender};

pub struct Start;

struct Loop;
struct Connect { socket: Option<TcpStream>, keep_alive: bool }

#[derive(Debug)]
struct Work { is_readable: bool, is_writable: bool }

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
        poll.registry().register(&mut socket, Token(0), Interest::READABLE).unwrap();

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
        if let Some(_) = envelope.message.downcast_ref::<Loop>() {
            self.poll.poll(&mut self.events, Some(Duration::from_millis(1))).unwrap();
            for event in self.events.iter() {
                match event.token() {
                    Token(0) => {
                        loop {
                            if let Ok((mut socket, _)) = self.socket.accept() {
                                //println!("connected: {}", self.counter + 1);
                                self.counter += 1;
                                let token = Token(self.counter);
                                self.poll.registry()
                                    .register(&mut socket, token,
                                              Interest::READABLE | Interest::WRITABLE)
                                    .unwrap();
                                let tag = format!("{}", self.counter);
                                sender.spawn(&tag, || Box::new(Connection::default()));
                                let connect = Connect { socket: Some(socket), keep_alive: true };
                                sender.send(&tag, Envelope::of(connect, ""));
                            } else {
                                break
                            }
                        }
                    },
                    token => {
                        let tag = format!("{}", token.0);
                        let work = Work { is_readable: event.is_readable(), is_writable: event.is_writable() };
                        sender.send(&tag, Envelope::of(work, ""));
                    }
                }
            }
            let me = sender.myself();
            sender.send(&me, Envelope::of(Loop, ""));
        } else if let Some(_) = envelope.message.downcast_ref::<Start>() {
            let me = sender.myself();
            sender.send(&me, Envelope::of(Loop, ""));
        }
    }
}

struct Connection {
    socket: Option<TcpStream>,
    keep_alive: bool,
    buffer: [u8; 1024],
    buffer_used: usize,
    recv_bytes: usize,
    can_read: bool,
    can_write: bool,
}

impl Default for Connection {
    fn default() -> Self {
        Connection {
            socket: None,
            keep_alive: true,
            buffer: [0 as u8; 1024],
            buffer_used: 0,
            recv_bytes: 0,
            can_read: false,
            can_write: false,
        }
    }
}

static RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: keep-alive\r\nContent-Length: 6\r\n\r\nhello\n";

impl AnyActor for Connection {
    fn receive(&mut self, mut envelope: Envelope, sender: &mut dyn AnySender) {
        if let Some(connect) = envelope.message.downcast_mut::<Connect>() {
            self.socket = connect.socket.take();
            self.keep_alive = connect.keep_alive;
        } else if self.socket.is_none() {
            let me = sender.myself();
            sender.send(&me, envelope);
        } else if let Some(work) = envelope.message.downcast_ref::<Work>() {
            //println!("work: {:?}", work);
            self.can_read = work.is_readable;
            self.can_write = self.can_write || work.is_writable;
            if self.can_read {
                //println!("connection {} is readable", sender.me());
                match self.socket.as_ref().unwrap().read(&mut self.buffer[self.buffer_used..]) {
                    Ok(0) | Err(_) => {
                        //println!("connection {} closed (read 0 bytes)", sender.me());
                        if !self.keep_alive {
                            self.socket = None;
                            let me = sender.myself();
                            sender.stop(&me);
                        }
                        return
                    },
                    Ok(n) => {
                        //println!("connection {} read {} bytes", sender.me(), n);
                        self.recv_bytes += n;
                        self.buffer_used += n;
                    }
                }
            }

            let rnrn = self.buffer.windows(4)
                .find(|w| (w[0] == b'\r') && (w[1] == b'\n') && (w[2] == b'\r') && (w[3] == b'\n'))
                .is_some();

            if rnrn {
                &self.buffer[0..RESPONSE.len()].copy_from_slice(RESPONSE.as_bytes());
                self.buffer_used = RESPONSE.len();
            }

            if self.can_write {
                //println!("connection {} is writable", sender.me());
                if self.buffer_used > 0 {
                    match self.socket.as_ref().unwrap().write_all(&self.buffer[0..self.buffer_used]) {
                        Ok(_) => {
                            //println!("connection {} written {} bytes", sender.me(), self.buffer_used);
                            self.buffer_used = 0;
                        },
                        _ => {
                            //println!("connection {} closed (write failed)", sender.me());
                            if !self.keep_alive {
                                self.socket = None;
                                let me = sender.myself();
                                sender.stop(&me);
                            }
                            return
                        }
                    }
                }
            }
        }
    }
}
