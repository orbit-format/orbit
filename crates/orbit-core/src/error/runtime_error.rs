use crate::ast::Span;

#[derive(Debug, thiserror::Error)]
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
