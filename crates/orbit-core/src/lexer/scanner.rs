use crate::{
    ast::Span,
    error::LexError,
    lexer::token::{Token, TokenKind},
};

pub struct Lexer<'a> {
    source: &'a str,
    offset: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, offset: 0 }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token<'a>>, LexError> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.peek_char() {
            match ch {
                ' ' | '\t' | '\x0c' => {
                    self.advance_char();
                }
                '\r' => {
                    let start = self.offset;
                    self.advance_char();
                    if self.peek_char() == Some('\n') {
                        self.advance_char();
                    }
                    let span = Span::new(start, self.offset);
                    tokens.push(Token {
                        kind: TokenKind::Newline,
                        span,
                    });
                }
                '\n' => {
                    let start = self.offset;
                    self.advance_char();
                    let span = Span::new(start, self.offset);
                    tokens.push(Token {
                        kind: TokenKind::Newline,
                        span,
                    });
                }
                '#' => tokens.push(self.lex_comment()?),
                '{' => tokens.push(self.symbol(TokenKind::LBrace)),
                '}' => tokens.push(self.symbol(TokenKind::RBrace)),
                '[' => tokens.push(self.symbol(TokenKind::LBracket)),
                ']' => tokens.push(self.symbol(TokenKind::RBracket)),
                ':' => tokens.push(self.symbol(TokenKind::Colon)),
                ',' => tokens.push(self.symbol(TokenKind::Comma)),
                '"' => tokens.push(self.lex_string()?),
                c if is_ident_start(c) => tokens.push(self.lex_ident_or_bool()?),
                c if c.is_ascii_digit()
                    || (c == '-' && self.peek_next_char().is_some_and(|n| n.is_ascii_digit())) =>
                {
                    tokens.push(self.lex_number()?)
                }
                other => {
                    let span = Span::new(self.offset, self.offset + other.len_utf8());
                    return Err(LexError::new(
                        format!("unexpected character '{other}'"),
                        span,
                    ));
                }
            }
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::new(self.offset, self.offset),
        });

        Ok(tokens)
    }

    fn symbol(&mut self, kind: TokenKind<'a>) -> Token<'a> {
        let start = self.offset;
        self.advance_char();
        let span = Span::new(start, self.offset);
        Token { kind, span }
    }

    fn lex_comment(&mut self) -> Result<Token<'a>, LexError> {
        let start = self.offset;
        self.advance_char();
        while let Some(ch) = self.peek_char() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.advance_char();
        }
        let span = Span::new(start, self.offset);
        let lexeme = &self.source[start..self.offset];
        Ok(Token {
            kind: TokenKind::Comment(lexeme),
            span,
        })
    }

    fn lex_string(&mut self) -> Result<Token<'a>, LexError> {
        let start = self.offset;
        self.advance_char();
        let mut value = String::new();
        while let Some(ch) = self.advance_char() {
            match ch {
                '"' => {
                    let span = Span::new(start, self.offset);
                    return Ok(Token {
                        kind: TokenKind::String(value),
                        span,
                    });
                }
                '\\' => {
                    let escaped = self.advance_char().ok_or_else(|| {
                        LexError::new("unterminated string escape", Span::new(start, self.offset))
                    })?;
                    value.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        other => other,
                    });
                }
                '\n' | '\r' => {
                    return Err(LexError::new(
                        "unterminated string literal",
                        Span::new(start, self.offset),
                    ));
                }
                other => value.push(other),
            }
        }

        Err(LexError::new(
            "unterminated string literal",
            Span::new(start, self.offset),
        ))
    }

    fn lex_ident_or_bool(&mut self) -> Result<Token<'a>, LexError> {
        let start = self.offset;
        self.advance_char();
        while let Some(ch) = self.peek_char() {
            if is_ident_part(ch) {
                self.advance_char();
            } else {
                break;
            }
        }
        let span = Span::new(start, self.offset);
        let lexeme = &self.source[start..self.offset];
        match lexeme {
            "true" => Ok(Token {
                kind: TokenKind::Bool(true),
                span,
            }),
            "false" => Ok(Token {
                kind: TokenKind::Bool(false),
                span,
            }),
            _ => Ok(Token {
                kind: TokenKind::Ident(lexeme),
                span,
            }),
        }
    }

    fn lex_number(&mut self) -> Result<Token<'a>, LexError> {
        let start = self.offset;
        self.advance_char();
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.advance_char();
            } else {
                break;
            }
        }
        if self.peek_char() == Some('.')
            && self.peek_next_char().is_some_and(|c| c.is_ascii_digit())
        {
            self.advance_char();
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    self.advance_char();
                } else {
                    break;
                }
            }
        }
        let span = Span::new(start, self.offset);
        let lexeme = &self.source[start..self.offset];
        Ok(Token {
            kind: TokenKind::Number(lexeme),
            span,
        })
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut iter = self.source[self.offset..].chars();
        iter.next()?;
        iter.next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_part(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit() || ch == '.' || ch == '-'
}

pub fn lex<'a>(source: &'a str) -> Result<Vec<Token<'a>>, LexError> {
    Lexer::new(source).tokenize()
}
