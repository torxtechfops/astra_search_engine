use std::{
    fmt,
    error::Error,
    process::exit
};

pub fn fatal_error<ErrorParam: Error>(text: &str, rust_error: &dyn Error) {
    if !rust_error.to_string().is_empty() {
        eprintln!("Fatal error occured: {} - {}", text, rust_error);
    } else {
        eprintln!("Fatal error occured: {}", text);
    };

    exit(1);
}

pub fn error<ErrorParam: Error>(text: &str, rust_error: &dyn Error) {
    if !rust_error.to_string().is_empty() {
        eprintln!("Nonfatal error occured: {} - {}", text, rust_error);
    } else {
        eprintln!("Nonfatal error occured: {}", text);
    };
}

#[derive(Debug, Clone)]
pub struct NoneError;

impl fmt::Display for NoneError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

impl std::error::Error for NoneError {}