#[allow(warnings)]
mod controller;
mod errors;
mod func;
mod request;
mod response;
mod server;

fn main() {
    server::init();
}
