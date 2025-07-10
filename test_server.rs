use crate::websocket::WebSocket;
use std::{thread, time};

pub fn init() -> bool {
    let port: &str = "1818";
    let host: &str = "192.168.0.102";
    let mut w: WebSocket = WebSocket::new_server(host, port);
    if let false = w.spawn() {
        println!("error creating server, wrong port?");
        false;
    }

    test_accept(&mut w);
    true
}

pub fn test_accept(w: &mut WebSocket) {
    w.set_single_thread_mode(true);
    loop {
        let ten_millis = time::Duration::from_millis(50);
        thread::sleep(ten_millis);
        w.accept_connections();
        thread::sleep(ten_millis);
        w.accept_messages();
    }
}
