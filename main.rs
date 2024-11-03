#[allow(warnings)]
mod controller;
mod errors;
mod func;
mod request;
mod response;
mod test_server;
mod websocket;

fn main() {
    test_server::init();
}
