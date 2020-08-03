extern crate log;
extern crate env_logger;
use log::debug;

use parsed::stream::ByteStream;
use parsed::http::{parse_http_request, Request, Header, Response, as_string};
use parsed::ws::{parse_frame, decode_frame_body, Frame};
use sha1::{Sha1, Digest};

fn get_header<'a>(headers: &'a Vec<Header>, name: &String) -> Option<&'a String> {
    headers.iter()
        .find(|h| &h.name == name)
        .map(|h| &h.value)
}

fn res_sec_websocket_accept(req_sec_websocket_key: &String) -> String {
    let mut hasher = Sha1::new();
    hasher.input(req_sec_websocket_key.to_owned() + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    base64::encode(hasher.result())
}

// https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers
fn handle(req: Request) -> Response {
    let connection = get_header(&req.headers, &"Connection".to_string())
        .map(|h| h.contains("Upgrade"))
        .unwrap_or_default();
    let upgrade = get_header(&req.headers, &"Upgrade".to_string())
        .map(|h| h.contains("websocket"))
        .unwrap_or_default();

    if connection && upgrade {
        let sec_websocket_accept =
            get_header(&req.headers, &"Sec-WebSocket-Key".to_string())
                .map(res_sec_websocket_accept)
                .unwrap_or_default();

        Response {
            protocol: "HTTP/1.1".to_string(),
            code: 101,
            message: "Switching Protocols".to_string(),
            headers: vec![
                Header {
                    name: "Upgrade".to_string(),
                    value: "websocket".to_string(),
                },
                Header {
                    name: "Connection".to_string(),
                    value: "Upgrade".to_string(),
                },
                Header {
                    name: "Sec-WebSocket-Accept".to_string(),
                    value: sec_websocket_accept,
                },
            ],
            content: vec![]
        }
    } else {
        Response {
            protocol: "HTTP/1.1".to_string(),
            code: 200,
            message: "OK".to_string(),
            headers: vec![
                Header { name: "Content-Type".to_string(), value: "text/html".to_string(), },
                Header { name: "Connection".to_string(), value: "keep-alive".to_string(), },
                Header { name: "Content-Length".to_string(), value: "6".to_string(), },
            ],
            content: "hello\n".as_bytes().to_vec(),
        }
    }
}

pub fn process(recv: &mut ByteStream, send: &mut ByteStream, is_open: &mut bool) {
    if let Some(req) = parse_http_request(recv) {
        debug!("request: {:?}", req);
        recv.pull();
        let res = handle(req);
        debug!("response: {:?}", res);
        let s: String = res.into();
        send.put(s.as_bytes());
    } else if let Some(frame) = parse_frame(recv) {
        debug!("ws frame: {:?}", frame);
        recv.pull();
        if frame.opcode != 0x08 { // opcode 0x08 represents CLOSE event
            let body = frame
                .mask.map(|mask| decode_frame_body(&frame.body, &mask))
                .unwrap_or_default();
            let body_as_string = as_string(body);
            debug!("ws frame body: '{}'", body_as_string);

            let res = Frame::text(&format!("ECHO: '{}'", body_as_string));
            debug!("ws response: {:?}", res);
            let v: Vec<u8> = res.into();
            send.put(v.as_slice());
        } else {
            debug!("ws close opcode (0x08) received");
            *is_open = false;
        }
    }
}
