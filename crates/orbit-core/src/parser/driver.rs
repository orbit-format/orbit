use crate::{
    ast::{AstNode, ObjectEntry, Span, ValueNode},
    error::{LexError, ParseError},
    lexer::{Token, TokenKind, lex},
    value::number::OrbitNumber,
};

use super::grammar::{Document, document};

pub struct ParseReport {
    pub document: Document,
    pub errors: Vec<ParseError>,
}

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
    last_consumed: Option<usize>,
}

impl<'a> Parser<'a> {
    pub fn from_source(source: &'a str) -> Result<Self, LexError> {
        let tokens = lex(source)?;
        Ok(Self {
            tokens,
            index: 0,
            last_consumed: None,
        })
    }

    pub fn parse_document(mut self) -> Result<Document, ParseError> {
        let span_start = self.tokens.first().map(|t| t.span.start).unwrap_or(0);
        let mut body = Vec::new();
        while !self.is_at_end() {
            body.push(self.parse_entry_or_block()?);
        }
        let span_end = self.tokens.last().map(|t| t.span.end).unwrap_or(span_start);
        Ok(document(body, Span::new(span_start, span_end)))
    }

    pub fn parse_document_with_recovery(mut self) -> ParseReport {
        let span_start = self.tokens.first().map(|t| t.span.start).unwrap_or(0);
        let mut body = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() {
            match self.parse_entry_or_block() {
                Ok(node) => body.push(node),
                Err(err) => {
                    errors.push(err);
                    self.synchronize();
                }
            }
        }
        let span_end = self.tokens.last().map(|t| t.span.end).unwrap_or(span_start);
        ParseReport {
            document: document(body, Span::new(span_start, span_end)),
            errors,
        }
    }

    fn parse_entry_or_block(&mut self) -> Result<AstNode, ParseError> {
        let (ident, ident_span) = self.consume_ident("expected identifier")?;
        if self.matches(|kind| matches!(kind, TokenKind::LBrace)) {
            self.parse_block(ident, ident_span)
        } else {
            self.expect(
                |kind| matches!(kind, TokenKind::Colon),
                "expected ':' after identifier",
            )?;
            let value = self.parse_value()?;
            let span = ident_span.union(value.span());
            Ok(AstNode::Entry {
                key: ident,
                value,
                span,
            })
        }
    }

    fn parse_block(&mut self, name: String, name_span: Span) -> Result<AstNode, ParseError> {
        let mut body = Vec::new();
        while !self.current_is(|kind| matches!(kind, TokenKind::RBrace)) {
            if self.is_at_end() {
                return Err(ParseError::new("unterminated block", name_span));
            }
            body.push(self.parse_entry_or_block()?);
        }
        let closing = self.expect(
            |kind| matches!(kind, TokenKind::RBrace),
            "expected '}' to close block",
        )?;
        let span = name_span.union(closing.span);
        Ok(AstNode::Block { name, body, span })
    }

    fn parse_value(&mut self) -> Result<ValueNode, ParseError> {
        match self.peek().kind.clone() {
            TokenKind::String(value) => {
                let token = self.advance().clone();
                Ok(ValueNode::String {
                    value,
                    span: token.span,
                })
            }
            TokenKind::Number(raw) => {
                let token = self.advance().clone();
                let number = parse_number_literal(raw, token.span)?;
                Ok(ValueNode::Number {
                    value: number,
                    span: token.span,
                })
            }
            TokenKind::Bool(value) => {
                let token = self.advance().clone();
                Ok(ValueNode::Bool {
                    value,
                    span: token.span,
                })
            }
            TokenKind::LBracket => self.parse_list(),
            TokenKind::LBrace => self.parse_object(),
            other => Err(ParseError::new(
                format!("unexpected token {} while parsing value", other.describe()),
                self.peek().span,
            )),
        }
    }

    fn parse_list(&mut self) -> Result<ValueNode, ParseError> {
        let open = self.advance().clone();
        let mut items = Vec::new();
        if self.matches(|kind| matches!(kind, TokenKind::RBracket)) {
            let close_span = self.previous().unwrap().span;
            return Ok(ValueNode::List {
                items,
                span: open.span.union(close_span),
            });
        }
        loop {
            let value = self.parse_value()?;
            items.push(value);
            if self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                if self.matches(|kind| matches!(kind, TokenKind::RBracket)) {
                    let close_span = self.previous().unwrap().span;
                    let span = open.span.union(close_span);
                    return Ok(ValueNode::List { items, span });
                }
                continue;
            }
            let close = self.expect(
                |kind| matches!(kind, TokenKind::RBracket),
                "expected ']' to close list",
            )?;
            let span = open.span.union(close.span);
            return Ok(ValueNode::List { items, span });
        }
    }

    fn parse_object(&mut self) -> Result<ValueNode, ParseError> {
        let open = self.advance().clone();
        let mut entries = Vec::new();
        if self.matches(|kind| matches!(kind, TokenKind::RBrace)) {
            let close_span = self.previous().unwrap().span;
            return Ok(ValueNode::Object {
                entries,
                span: open.span.union(close_span),
            });
        }
        loop {
            let (key, key_span) = self.consume_ident("expected key inside object")?;
            self.expect(
                |kind| matches!(kind, TokenKind::Colon),
                "expected ':' after key in object",
            )?;
            let value = self.parse_value()?;
            let entry_span = key_span.union(value.span());
            entries.push(ObjectEntry {
                key,
                value,
                span: entry_span,
            });

            if self.matches(|kind| matches!(kind, TokenKind::Comma)) {
                if self.matches(|kind| matches!(kind, TokenKind::RBrace)) {
                    let close_span = self.previous().unwrap().span;
                    let span = open.span.union(close_span);
                    return Ok(ValueNode::Object { entries, span });
                }
                continue;
            }

            let close = self.expect(
                |kind| matches!(kind, TokenKind::RBrace),
                "expected '}' to close object",
            )?;
            let span = open.span.union(close.span);
            return Ok(ValueNode::Object { entries, span });
        }
    }

    fn consume_ident(&mut self, message: &str) -> Result<(String, Span), ParseError> {
        let token = self.expect(|kind| matches!(kind, TokenKind::Ident(_)), message)?;
        if let TokenKind::Ident(raw) = token.kind {
            Ok((raw.to_string(), token.span))
        } else {
            unreachable!()
        }
    }

    fn matches<F>(&mut self, predicate: F) -> bool
    where
        F: Fn(&TokenKind<'a>) -> bool,
    {
        if predicate(&self.peek().kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect<F>(&mut self, predicate: F, message: &str) -> Result<&Token<'a>, ParseError>
    where
        F: Fn(&TokenKind<'a>) -> bool,
    {
        if predicate(&self.peek().kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(message, self.peek().span))
        }
    }

    fn current_is<F>(&mut self, predicate: F) -> bool
    where
        F: Fn(&TokenKind<'a>) -> bool,
    {
        predicate(&self.peek().kind)
    }

    fn advance(&mut self) -> &Token<'a> {
        self.skip_trivia();
        let idx = self.index;
        self.last_consumed = Some(idx);
        if self.index < self.tokens.len() - 1 {
            self.index += 1;
            self.skip_trivia();
        }
        &self.tokens[idx]
    }

    fn peek(&mut self) -> &Token<'a> {
        self.skip_trivia();
        &self.tokens[self.index]
    }

    fn previous(&self) -> Option<&Token<'a>> {
        self.last_consumed.map(|idx| &self.tokens[idx])
    }

    fn skip_trivia(&mut self) {
        while self.index < self.tokens.len() && self.tokens[self.index].is_trivia() {
            self.index += 1;
        }
        if self.index >= self.tokens.len() {
            self.index = self.tokens.len().saturating_sub(1);
        }
    }

    fn is_at_end(&mut self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn synchronize(&mut self) {
        if matches!(self.peek().kind, TokenKind::Eof) {
            return;
        }
        self.advance();
        while !self.is_at_end() {
            match self.peek().kind {
                TokenKind::Ident(_) | TokenKind::RBrace => break,
                _ => {
                    self.advance();
                }
            }
        }
    }
}

fn parse_number_literal(raw: &str, span: Span) -> Result<OrbitNumber, ParseError> {
    if raw.contains(['.', 'e', 'E']) {
        raw.parse::<f64>()
            .map(OrbitNumber::Float)
            .map_err(|_| ParseError::new("invalid float literal", span))
    } else {
        raw.parse::<i64>()
            .map(OrbitNumber::Integer)
            .map_err(|_| ParseError::new("invalid integer literal", span))
    }
}
