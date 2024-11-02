use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct InvalidHTTP;

impl Display for InvalidHTTP {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Invalid HTTP")
    }
}

impl Error for InvalidHTTP {}
