use crate::websocket::WebSocket;
use std::net::TcpListener;
use std::process;

pub fn init() {
    let port: &str = "1818";
    let host: &str = "192.168.0.108";
    let mut w: WebSocket = WebSocket::new_server(host, port);
    w.spawn();
    w.accept();
}
