use std::net::TcpListener;
use std::process;
use crate::controller::handle_stream;


fn run_server(listener: TcpListener) {
    for stream in listener.incoming() {
        let s = stream.unwrap();
        // For now/testing purposes, this will block the main thread for each new connection
        println!("New Client Connection");
        handle_stream(s);
    }
}

pub fn init() {
    let port: &str = "1818";
    let host: &str = "192.168.0.103";
    let hn: &str = &format!("{}:{}", &host, &port);

    let listener = TcpListener::bind(&hn).unwrap();
    println!("Started on {}:{}", host, port);
    println!("Proccess ID: {}", process::id());
    run_server(listener);
}
