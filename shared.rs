use std::collections::HashMap;
use std::io::{prelude::*, BufReader};

use std::fs::read;
use std::io::Read;
use std::net::TcpStream;


#[derive(Debug)]
pub enum RequestType {
    Http,
    WebSocket
}

#[derive(Debug)]
pub struct Request {
    pub _type: RequestType,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl Request {
    pub fn new_from_stream(stream: &TcpStream) -> Self {
        let mut buf_r = BufReader::new(stream);
        let mut data_itr = buf_r.by_ref().lines().map(|result| result.unwrap());

        let mut headers: HashMap<String, String> = HashMap::new();
        let mut body = String::new();
        let mut request_type: RequestType = RequestType::Http;

        if let Some(req_line) = data_itr.next() {
            headers.insert(String::from("RequestLine"), req_line);
        }

        let mut content_length = 0;

        while let Some(s) = data_itr.next() {
            if s.is_empty() {
                break;
            }

            let mut parts = s.splitn(2, ':');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("").trim().to_string();
                if key.trim().eq_ignore_ascii_case("Content-Length") {
                    content_length = value.parse::<usize>().unwrap_or(0);
                }

                if key.trim().to_ascii_lowercase().contains("websocket") {
                    request_type = RequestType::WebSocket;
                }
                headers.insert(key.trim().to_string(), value);
            }
        }

        if content_length > 0 {
            let mut body_buffer = vec![0; content_length];
            buf_r.read_exact(&mut body_buffer).expect("Couldn't read body");
            body = String::from_utf8(body_buffer).expect("Couldn't decode body");
        }

        Request {
            _type: request_type,
            headers: headers,
            body: body,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub headers: HashMap<String, String>,
    pub body: String
}

impl Response {
    pub fn new() -> Self {
        Response {
            headers: HashMap::new(),
            body: String::new()
        }
    }

    pub fn new_no_body(headers: HashMap<String, String>) -> Self {
        Response {
            headers,
            body: String::new()
        }
    }

    pub fn headers_to_string(&self) -> String {
        let mut h_str = String::new();
        let status_line = self.headers.get("StatusLine").unwrap();
        h_str.push_str(status_line);
        h_str.push_str("\r\n");
        for (key, value) in self.headers.iter() {
            if key == "StatusLine" {
                continue
            }
            h_str.push_str(&key);
            h_str.push_str(": ");
            h_str.push_str(&value);
            h_str.push_str("\r\n");
        }
        h_str.push_str("\r\n");
        h_str
    }
}