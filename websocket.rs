use crate::controller::{Frame, WebSocketController};
use std::net::TcpListener;
pub struct WebSocket<'a> {
    pub host: &'a str,
    pub port: &'a str,
    pub is_active: bool,
    pub run_single_threaded: bool,
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
            run_single_threaded: false,
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
                println!("Started on {}", hn);
                return true;
            }
            Err(_) => {
                // probably panic here
                self.listener = None;
                return false;
            }
        }
    }

    pub fn set_single_thread_mode(&mut self, b: bool) {
        self.run_single_threaded = b;
        if let Some(l) = &self.listener {
            l.set_nonblocking(b).unwrap(); // TODO error handling
        }
    }

    pub fn accept_connections(&mut self) {
        if let Some(l) = &self.listener {
            for s in l.incoming() {
                match s {
                    Ok(stream) => {
                        let new_id: u64 = (self.connections.len() + 1).try_into().unwrap();
                        let c = WebSocketController::init(stream, new_id);
                        match c {
                            Ok(controller) => {
                                println!("new client connection: {}", controller.id);
                                self.connections.push(controller);
                            }
                            Err(_) => (),
                        }
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
            if let Ok(_) = c.receive_into_buff() {
                for f in c.frame_buff.drain(..) {
                    println!("new message received from client {}", c.id);
                    dbg!(f);
                }
            }
        }
    }

    fn close_connections(&mut self) {
        self.connections.retain_mut(|c| c.verify());
    }

    pub fn alert_all(&mut self, message: String) {
        for c in &mut self.connections {
            c.send(message.clone());
        }
    }
}
