extern crate log;
extern crate env_logger;
use log::debug;

use crate::client::Client;
use std::time::Duration;

mod client;

fn main() {
    env_logger::init();

    let mut client = Client::new();

    let target = "127.0.0.1:9000"; // nc -l 9000
    // let target = "216.58.215.78:80"; // ping google.com

    let _id = client.connect(target).unwrap();

    // client.put(target, b"hello there!");
    client.put(target, b"GET / HTTP/1.1\r\n\r\n");

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
