use std::{io::Write, net::TcpStream};

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
