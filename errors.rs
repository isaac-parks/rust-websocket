use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct WebSocketError;

impl Display for WebSocketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Invalid Websocket")
    }
}

impl Error for WebSocketError {}
