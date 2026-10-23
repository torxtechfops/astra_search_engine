use std::fmt;

use crate::tcp::BUFFER_SIZE;

#[derive(Debug, Clone)]
pub struct StreamTooLongError;

impl fmt::Display for StreamTooLongError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stream too long to be written to vector of {} bytes", BUFFER_SIZE)
    }
}

impl std::error::Error for StreamTooLongError {}

#[derive(Debug, Clone)]
pub struct UnknownRequestTypeError;

impl fmt::Display for UnknownRequestTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown request type")
    }
}

impl std::error::Error for UnknownRequestTypeError {}
