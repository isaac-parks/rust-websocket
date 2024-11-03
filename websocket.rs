use crate::controller::WebSocketController;
use std::net::TcpListener;
pub struct WebSocket<'a> {
    pub host: &'a str,
    pub port: &'a str,
    pub is_active: bool,
    listener: Option<TcpListener>,
    connections: Vec<WebSocketController>,
}

impl<'a> WebSocket<'a> {
    pub fn new_server(host: &'a str, port: &'a str) -> Self {
        WebSocket {
            host,
            port,
            listener: None,
            connections: Vec::new(),
            is_active: false,
        }
    }

    pub fn spawn(&mut self) -> bool {
        let hn: &str = &format!("{}:{}", &self.host, self.port);
        let listener = TcpListener::bind(&hn);
        match listener {
            Ok(l) => {
                self.listener = Some(l);
                self.is_active = true;
                println!("Started on:{}", hn);
                return true;
            }
            Err(_) => {
                // probably panic here
                self.listener = None;
                return false;
            }
        }
    }

    pub fn accept_connections(&mut self) {
        if let Some(l) = &self.listener {
            l.set_nonblocking(true).unwrap();
            for s in l.incoming() {
                match s {
                    Ok(stream) => {
                        let new_id: u64 = (self.connections.len() + 1).try_into().unwrap();
                        let mut c = WebSocketController::new(stream, new_id);
                        if !c.handshake() {
                            println!("connection refused, invalid handshake");
                        }
                        println!("new client connection: {}", c.id);
                        self.connections.push(c);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        }
    }

    pub fn accept_messages(&mut self) {
        for c in &mut self.connections {
            c.receive_messages();
            for f in c.frame_buff.drain(..) {
                println!("new message received from client {}", c.id);
                dbg!(f);
            }
        }
    }
}
