use std::net::TcpListener;
use crate::controller::handle_stream;


fn run_server(listener: TcpListener) {
    for stream in listener.incoming() {
        let s = stream.unwrap();
        // For now/testing purposes, this will block the main thread for each new connection
        handle_stream(s)
    }
}

pub fn init() {
    let port: &str = "1818";
    let host: &str = "127.0.0.1";
    let hn: &str = &format!("{}:{}", &host, &port);

    let listener = TcpListener::bind(&hn).unwrap();
    println!("Started on {}:{}", host, port);
    run_server(listener);
}
