use crate::websocket::WebSocket;
use std::net::TcpListener;
use std::process;
use std::{thread, time};

pub fn init() {
    let port: &str = "1818";
    let host: &str = "192.168.0.108";
    let mut w: WebSocket = WebSocket::new_server(host, port);
    w.spawn();

    test_accept(&mut w);
}

pub fn test_accept(w: &mut WebSocket) {
    loop {
        let ten_millis = time::Duration::from_millis(1);
        thread::sleep(ten_millis);
        w.accept_connections();
        let ten_millis = time::Duration::from_millis(1);
        thread::sleep(ten_millis);
        w.accept_messages();
    }
}
