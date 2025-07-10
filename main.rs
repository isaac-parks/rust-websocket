use std::process::ExitCode;

#[allow(warnings)]
mod controller;
mod errors;
mod func;
mod request;
mod response;
mod test_server;
mod websocket;

fn main() -> ExitCode{
    if (!test_server::init()) {
        return ExitCode::from(1);
    }

    return ExitCode::from(0);
}
