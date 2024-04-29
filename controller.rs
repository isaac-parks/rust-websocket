use std::{io::Write, net::TcpStream};

use crate::shared::{Request, RequestType};
use websocket::{WebSocketController, ConnectionStatus};
use http::HttpController;


fn make_controller(stream: TcpStream) -> Box<dyn Controller> {
    let init_req = Request::new_from_stream(&stream);
    match init_req._type {
        RequestType::Http => Box::new(HttpController {request: init_req, stream}),
        RequestType::WebSocket => Box::new(WebSocketController {request: init_req, stream, conn_status:ConnectionStatus::Open}),
    }
}

pub fn handle_stream(stream: TcpStream) {
    let mut c = make_controller(stream);
    c.handle();
}

pub fn write_to_stream(content: String, stream: &mut TcpStream) {
    let bytes = content.as_bytes();
    stream.write_all(bytes).unwrap();
    if let Err(_) = stream.flush() {
        println!("Error occured sending response.");
    }
}

trait Controller {
    fn handle(&mut self);
}

mod websocket {
    use sha1::{Digest, Sha1};
    use base64;
    use std::io::Write;
    use std::{collections::HashMap, io::Read, net::TcpStream};
    use std::time::Duration;
    use super::Controller;
    use super::write_to_stream;
    use crate::shared::Response;

    #[derive(PartialEq)]
    pub enum ConnectionStatus {
        Open,
        Closed
    }
    #[derive(Debug)]
    enum Opcode {
        Cont,
        Text,
        Binary,
        NonCtrl,
        ConnClosed,
        Ping,
        Pong,
        Ctrl
    }
    #[derive(Debug)]
    struct Frame {
        is_final: bool,
        opcode: Opcode,
        is_masked: bool,
        masking_key: u32,
        payload_len: u8,
        payload: String
    }

    impl Frame {
        fn new(stream: &mut TcpStream) -> Option<Self> {
            let mut op_buff: [u8; 2] = [0; 2];
            let read_op = stream.read_exact(&mut op_buff);

            if let Err(_) = read_op {
                return None;
            }

            // First Byte
            let is_final = op_buff[0] & 0b10000000 != 0; // First bit is whether this is the final segment of the message
            let opcode = match op_buff[0] & 0b00001111 { // next 4 bits are the opcode
                0x1 => Opcode::Text,
                0x2 => Opcode::Binary,
                0x9 => Opcode::Ping,
                0xA => Opcode::Pong,
                _ => Opcode::Ctrl
            };

            // Second Byte
            let is_masked = op_buff[1] & 0b10000000 != 0; // First bit is whether payload is masked
            let payload_len = op_buff[1] & 0b01111111;


            // 3rd 4th 5th and 6th byte is masking-key
            let mut mask_key: [u8; 4] = [0; 4];
            if is_masked {
                stream.read_exact(&mut mask_key).unwrap();
            }

            // Rest will be payload
            let mut payload_buff = vec![0u8; payload_len as usize];
            stream.read_exact(&mut payload_buff).unwrap();
            
            if is_masked { // Demask payload if needed
                for i in 0..payload_len as usize {
                    payload_buff[i] ^= mask_key[i % 4];
                }
            }

            let mut payload = String::new();
            let payload_str = String::from_utf8(payload_buff.to_vec());
            if let Ok(s) = payload_str {
                payload.push_str(&s)
            }

            let frame = Frame {
                is_final,
                opcode,
                is_masked,
                masking_key: 0,
                payload_len,
                payload
            };

            dbg!(&frame);
            Some(frame)
        }
    }
    pub struct WebSocketController {
        pub request: super::Request,
        pub stream: super::TcpStream,
        pub conn_status: super::ConnectionStatus
    }

    impl Controller for WebSocketController {
        fn handle(&mut self) {
            let _ = self.stream.set_read_timeout(Some(Duration::from_millis(50)));
            if self.check_handshake() {
                self.do_handshake()
            }
            loop {
                self.incoming_frame();
            }
        }
    }

    impl WebSocketController {
        fn check_handshake(&self) -> bool {
            let request_line = self.request.headers.get("RequestLine");
            let upgrade_header = self.request.headers.get("Upgrade");
            if let Some(h) = upgrade_header {
                if h == "websocket" && request_line.unwrap_or(&String::new()).contains("GET") {
                    return true;
                }
            }
    
            false
        }
    
        fn do_handshake(&mut self) {
            let mut resp_headers: HashMap<String, String> = HashMap::new();
    
            let client_key = self.request.headers.get("Sec-WebSocket-Key").unwrap().clone(); // TODO
            let resp_key = self.make_hs_key(&client_key);
    
            resp_headers.insert("StatusLine".to_string(), "HTTP/1.1 101 Switching Protocols".to_string());
            resp_headers.insert("Upgrade".to_string(), "websocket".to_string());
            resp_headers.insert("Connection".to_string(), "Upgrade".to_string());
            resp_headers.insert("Sec-WebSocket-Accept".to_string(), resp_key);
            
            let resp = Response::new_no_body(resp_headers);
            let resp_str = resp.headers_to_string();
    
            write_to_stream(resp_str, &mut self.stream);
        }
    
        fn make_hs_key(&self, request_key: &String) -> String {
            let websocket_key: String = String::from("258EAFA5-E914-47DA-95CA-C5AB0DC85B11"); // Default websocket key
            let mut client_key: String = request_key.clone();
            client_key.push_str(&websocket_key); // Concat the websocket key onto the key sent from client
    
            let mut hasher = Sha1::new(); // Create Sha1 hash of the concat key
            hasher.update(client_key.as_bytes());
            let hashed_key = hasher.finalize();
    
            let base64_key: String = base64::encode(hashed_key); // Finalize key with base64
    
            base64_key
        }
    
        fn incoming_frame(&mut self) {
            let frame = Frame::new(&mut self.stream);
            if let Some(f) = frame {
                if f.payload == "pingme" { // Temp for testing ping 
                    self.send_ping();
                }
                println!("{}", f.payload);
            }
        }

        fn send_ping(&mut self) -> bool {
            let op_byte = 0b10001001;
            let payload_byte = 0b00000000;
            let ping: [u8; 2] = [op_byte, payload_byte];
            let r = self.stream.write_all(&ping);
            if let Ok(()) = r {
                if let Ok(_) = self.stream.flush() {
                    return true;
                }
            }

            false
        }
    }
}

mod http {
    use std::net::TcpStream;
    use super::Controller;
    use crate::shared::Request;

    pub struct HttpController {
        pub request: Request,
        pub stream:  TcpStream
    }

    impl Controller for HttpController {
        fn handle(&mut self) {
        }
    }
}