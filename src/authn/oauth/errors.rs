use std::fmt::Display;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum TokenValidationError {
    InvalidHeader,
    MissingKid,
    KeyNotFound,
    InvalidKey,
    TokenDecode,
}

impl Display for TokenValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OAuth Token Validation failed ~> {self:?}")
    }
}

impl Error for TokenValidationError {}