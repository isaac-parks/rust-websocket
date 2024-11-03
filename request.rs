use std::collections::HashMap;
use std::io::{prelude::*, BufReader};

use crate::errors::WebSocketError;
use std::io::Read;
use std::net::TcpStream;

#[derive(Debug)]
pub struct Request {
    pub is_valid: bool,
    pub headers: HashMap<String, String>,
    // pub uri: String,
    pub body: String,
}

impl Request {
    fn parse_headers(s: &mut TcpStream) -> (HashMap<String, String>, bool, usize) {
        let mut is_valid = false;
        let mut reader = BufReader::new(s);
        let mut itr = reader.by_ref().lines().map(|result| result.unwrap());
        let mut headers: HashMap<String, String> = HashMap::new();
        if let Some(req_line) = itr.next() {
            headers.insert(String::from("RequestLine"), req_line);
        }

        let mut content_length = 0;

        while let Some(s) = itr.next() {
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
                    is_valid = true;
                }
                headers.insert(key.trim().to_string(), value);
            }
        }

        (headers, is_valid, content_length)
    }
    fn parse_body(reader: &mut TcpStream, content_length: usize) -> String {
        if content_length <= 0 {
            return String::new();
        }
        let mut body_buffer = vec![0; content_length];
        reader
            .read_exact(&mut body_buffer)
            .expect("Couldn't read body");

        String::from_utf8(body_buffer).expect("Couldn't decode body")
    }
    pub fn new_from_stream(stream: &mut TcpStream) -> Result<Self, WebSocketError> {
        let (headers, is_valid, content_length) = Self::parse_headers(stream);
        let body = Self::parse_body(stream, content_length);

        Ok(Request {
            is_valid,
            headers,
            body,
        })
    }
    pub fn new_empty() -> Self {
        Request {
            headers: HashMap::new(),
            is_valid: false,
            body: String::new(),
        }
    }
}
