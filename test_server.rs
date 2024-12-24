use crate::websocket::WebSocket;
use std::{thread, time};

pub fn init() {
    let port: &str = "1818";
    let host: &str = "192.168.0.105";
    let mut w: WebSocket = WebSocket::new_server(host, port);
    if let true = w.spawn() {
        test_accept(&mut w);
    }

    println!("error creating server")
}

pub fn test_accept(w: &mut WebSocket) {
    w.set_single_thread_mode(true);
    loop {
        let ten_millis = time::Duration::from_millis(50);
        thread::sleep(ten_millis);
        w.accept_connections();
        let ten_millis = time::Duration::from_millis(50);
        thread::sleep(ten_millis);
        w.accept_messages();
    }
}
