extern crate log;
extern crate env_logger;
use log::debug;
use chrono::{DateTime, Utc};

use parsed::stream::ByteStream;
use parsed::http::{parse_http_request, Request, Header, Response};

fn handle(_: Request) -> Response {
    let now: DateTime<Utc> = Utc::now();

    Response {
        protocol: "HTTP/1.1".to_string(),
        code: 200,
        message: "OK".to_string(),
        headers: vec![
            Header { name: "Content-Length".to_string(), value: "6".to_string(), },
            Header { name: "Content-Type".to_string(), value: "text/plain; charset=utf-8".to_string(), },
            Header { name: "Date".to_string(), value: now.to_rfc2822(), },
        ],
        content: "hello\n".as_bytes().to_vec(),
    }
}

pub fn process(recv: &mut ByteStream, send: &mut ByteStream) {
    if let Some(req) = parse_http_request(recv) {
        debug!("request: {:?}", req);
        recv.pull();
        let res = handle(req);
        debug!("response: {:?}", res);
        let s: String = res.into();
        send.put(s.as_bytes());
    }
}
