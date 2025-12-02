use crate::ast::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}

impl<'a> Token<'a> {
    pub fn is_trivia(&self) -> bool {
        matches!(self.kind, TokenKind::Newline | TokenKind::Comment(_))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'a> {
    Ident(&'a str),
    String(String),
    Number(&'a str),
    Bool(bool),
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Comma,
    Newline,
    Comment(&'a str),
    Eof,
}

impl<'a> TokenKind<'a> {
    pub fn describe(&self) -> &'static str {
        match self {
            TokenKind::Ident(_) => "identifier",
            TokenKind::String(_) => "string",
            TokenKind::Number(_) => "number",
            TokenKind::Bool(_) => "boolean",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::Colon => ":",
            TokenKind::Comma => ",",
            TokenKind::Newline => "newline",
            TokenKind::Comment(_) => "comment",
            TokenKind::Eof => "end of file",
        }
    }
}
