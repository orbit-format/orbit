use crate::ast::Span;

#[derive(Debug, thiserror::Error)]
#[error("parse error at byte range {span:?}: {message}")]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}
