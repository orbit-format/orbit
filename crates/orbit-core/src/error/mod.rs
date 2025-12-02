pub mod lex_error;
pub mod parse_error;
pub mod runtime_error;

pub use self::lex_error::LexError;
pub use self::parse_error::ParseError;
pub use self::runtime_error::RuntimeError;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error(transparent)]
    Lex(#[from] LexError),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}
