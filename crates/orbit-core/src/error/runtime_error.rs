use crate::ast::Span;
use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
#[error("runtime error at byte range {span:?}: {message}")]
pub struct RuntimeError {
    pub message: String,
    pub span: Span,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}
