use crate::ast::Span;

#[derive(Debug, thiserror::Error)]
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
