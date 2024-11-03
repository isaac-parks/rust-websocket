use crate::controller::WebSocketController;
use std::net::TcpListener;
pub struct WebSocket<'a> {
    pub host: &'a str,
    pub port: &'a str,
    pub is_active: bool,
    listener: Option<TcpListener>,
    controller: Option<WebSocketController>,
}

impl<'a> WebSocket<'a> {
    pub fn new_server(host: &'a str, port: &'a str) -> Self {
        WebSocket {
            host,
            port,
            listener: None,
            controller: None,
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
                return true;
            }
            Err(_) => {
                // probably panic here
                self.listener = None;
                return false;
            }
        }
    }

    pub fn accept(&mut self) {
        if let Some(l) = &self.listener {
            for s in l.incoming() {
                match s {
                    Ok(stream) => {
                        let mut c = WebSocketController::new(stream);
                        c.receive().unwrap();
                        self.controller = Some(c);
                    }
                    Err(_) => self.controller = None,
                }
            }
        }
    }
}
