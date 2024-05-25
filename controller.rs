use std::{io::Write, net::TcpStream};

use crate::shared::{Request, RequestType};
use websocket::{WebSocketController, ConnectionStatus};
use http::HttpController;


pub fn handle_stream(stream: TcpStream) {
    let mut c = make_controller(stream);
    c.handle();
}

fn make_controller(stream: TcpStream) -> Box<dyn Controller> {
    let init_req = Request::new_from_stream(&stream);
    match init_req._type {
        RequestType::Http => Box::new(HttpController {request: init_req, stream}),
        RequestType::WebSocket => Box::new(WebSocketController {request: init_req, stream, conn_status:ConnectionStatus::Open}),
    }
}

pub fn write_string_to_stream(content: String, stream: &mut TcpStream) {
    let bytes = content.as_bytes();
    stream.write_all(bytes).unwrap();
    if let Err(_) = stream.flush() {
        println!("Error writing to stream");
    }
}

pub fn write_bytes_to_stream(bytes: &[u8], stream: &mut TcpStream) {
    stream.write_all(bytes).unwrap();
    if let Err(_) = stream.flush() {
        println!("Error writing to stream");
    }
}

trait Controller {
    fn handle(&mut self);
}

mod websocket {
    use sha1::{Digest, Sha1};
    use base64;
    use std::{collections::HashMap, io::Read, net::TcpStream};
    use std::time::Duration;
    use super::{write_bytes_to_stream, write_string_to_stream, Controller};
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
        fn parse_op_code(byte: [u8; 1]) -> (bool, Option<Opcode>) {
            let is_final =  byte[0] & 0b10000000 != 0; // First bit is whether this is the final segment of the message
            let opcode = match byte[0] & 0b00001111 { // next 4 bits are the opcode
                0x1 => Some(Opcode::Text),
                0x2 => Some(Opcode::Binary),
                0x9 => Some(Opcode::Ping),
                0xA => Some(Opcode::Pong),
                _ => None
            };

            (is_final, opcode)
        }

        fn parse_payload_len(byte: [u8; 1]) -> (bool, u8){
            let is_masked = byte[0] & 0b10000000 != 0; // First bit is whether payload is masked
            let payload_len = byte[0] & 0b01111111;

            (is_masked, payload_len)
        }

        fn read_mask_key_into_buff(buff: &mut [u8; 4], stream: &mut TcpStream) {
            stream.read_exact(buff).unwrap();
        }

        fn read_payload(payload_buff: &mut Vec<u8>, is_masked: bool, mask_key: [u8; 4], stream: &mut TcpStream) -> String{
            stream.read_exact(payload_buff).unwrap();
            
            if is_masked {
                for i in 0..payload_buff.len() {
                    payload_buff[i] ^= mask_key[i % 4];
                }
            }

            let mut payload = String::new();
            let payload_str = String::from_utf8(payload_buff.to_vec());
            if let Ok(s) = payload_str {
                payload.push_str(&s)
            }

            payload
        }

        fn parse_frame_buffer(stream: &mut TcpStream) -> Option<Frame> {
            // First byte is whether this is the final segment of the message plus the opcode
            let mut opcode_buff: [u8; 1] = [0; 1];
            let read_op = stream.read_exact(&mut opcode_buff);
            if let Err(_) = read_op {
                return None;
            }
            let (is_final, opcode) = Frame::parse_op_code(opcode_buff);
            if let None = opcode {
                return None;
            }

            // Second byte is whether the payload is masked, and the payload length
            let mut payload_len_buff: [u8; 1] = [0; 1];
            let read_payload_len = stream.read_exact(&mut payload_len_buff);
            if let Err(_) = read_payload_len {
                return None;
            }
            let (is_masked, payload_len) = Frame::parse_payload_len(payload_len_buff);


            // 3rd 4th 5th and 6th byte is masking key
            let mut mask_key_buff: [u8; 4] = [0; 4];
            if is_masked {
                Frame::read_mask_key_into_buff(&mut mask_key_buff, stream);
            }

            // Rest will be payload
            let mut payload_buff = vec![0u8; payload_len as usize];
            let payload = Frame::read_payload(&mut payload_buff, is_masked, mask_key_buff, stream);

            let frame = Frame {
                is_final,
                opcode: opcode.unwrap(),
                is_masked,
                masking_key: 0,
                payload_len,
                payload
            };

            dbg!(&frame);
            Some(frame)
        }

        fn new(stream: &mut TcpStream) -> Option<Self> {
            Frame::parse_frame_buffer(stream)
        }
    }
    pub struct WebSocketController {
        pub request: super::Request,
        pub stream: super::TcpStream,
        pub conn_status: super::ConnectionStatus
    }

    impl Controller for WebSocketController {
        fn handle(&mut self) {
            self.stream.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
            if self.verify_handshake() {
                self.do_handshake()
            }
            loop {
                let frame: Option<Frame> = self.incoming_frame();
                if let Some(f) = frame {
                    // TODO: Probably have a connection status enum to specify whetehr to hangup the connection.
                    self.handle_frame(f);
                    if self.conn_status == ConnectionStatus::Closed {
                        break;
                    }
                }
            }
        }
    }

    impl WebSocketController {
        fn verify_handshake(&self) -> bool {
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
    
            let client_key = self.request.headers.get("Sec-WebSocket-Key").unwrap().clone();
            let resp_key = self.make_hs_key(&client_key);
    
            resp_headers.insert("StatusLine".to_string(), "HTTP/1.1 101 Switching Protocols".to_string());
            resp_headers.insert("Upgrade".to_string(), "websocket".to_string());
            resp_headers.insert("Connection".to_string(), "Upgrade".to_string());
            resp_headers.insert("Sec-WebSocket-Accept".to_string(), resp_key);
            
            let resp = Response::new_no_body(resp_headers);
            let resp_str = resp.headers_to_string();
    
            write_string_to_stream(resp_str, &mut self.stream);
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
    
        fn incoming_frame(&mut self) -> Option<Frame> {
            Frame::new(&mut self.stream)
        }

        fn handle_frame(&mut self, frame: Frame) -> bool {
            if frame.payload == "pingme" { // Temp for testing ping 
                self.send_ping();
            }

            if frame.payload == "closeconnection" {
                self.close_connection();
                return false;
            }
            println!("{}", frame.payload);
            true
        }

        fn send_ping(&mut self) -> bool {
            let op_byte = 0b10001001;
            let payload_byte = 0b00000000;
            let ping: [u8; 2] = [op_byte, payload_byte];
            write_bytes_to_stream(&ping, &mut self.stream);

            false
        }

        fn close_connection(&mut self) -> bool { // TODO support more status codes
            let op_byte = [0b10001000];
            let payload_len_byte = [0b00000010];
            let payload: [u8; 2] = [0b00000011, 0b11101000]; // 1000 status code - normal closer
            write_bytes_to_stream(&op_byte, &mut self.stream);
            write_bytes_to_stream(&payload_len_byte, &mut self.stream);
            write_bytes_to_stream(&payload, &mut self.stream);

            self.conn_status = ConnectionStatus::Closed;

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