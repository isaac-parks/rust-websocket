#[allow(warnings)]
mod controller;
mod errors;
mod func;
mod request;
mod response;
mod server;
mod websocket;

fn main() {
    server::init();
}
