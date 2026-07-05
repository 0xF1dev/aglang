use std::error::Error;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum SyntaxError {
    InvalidToken,
    InvalidArguments,
    InvalidSource,
    InvalidDestination,
    InvalidStatement,
    ChainedOperations,
}
impl Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SyntaxError({self:?})")
    }
}
impl Error for SyntaxError {}

pub fn error<D>(error: Box<dyn Error>, statement_number: u32, msg: D) where D: Display {
    eprintln!("Error {error} on statement {}: {msg}", statement_number + 1);
    std::process::exit(1)
}
