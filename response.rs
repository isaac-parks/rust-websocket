use std::collections::HashMap;
use std::io::{prelude::*, BufReader};

use std::io::Read;
use std::net::TcpStream;

#[derive(Debug)]
pub struct Response {
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    pub fn new() -> Self {
        Response {
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn new_no_body(headers: HashMap<String, String>) -> Self {
        Response {
            headers,
            body: String::new(),
        }
    }

    pub fn headers_to_string(&self) -> String {
        let mut h_str = String::new();
        let status_line = self.headers.get("StatusLine").unwrap();
        h_str.push_str(status_line);
        h_str.push_str("\r\n");
        for (key, value) in self.headers.iter() {
            if key == "StatusLine" {
                continue;
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
