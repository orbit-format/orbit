use crate::ast::Span;
use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
#[error("lex error at byte range {span:?}: {message}")]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl LexError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}
