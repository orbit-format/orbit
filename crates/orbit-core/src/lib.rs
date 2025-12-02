pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod serializer;
pub mod value;

pub use crate::ast::{AstNode, ObjectEntry, Span, ValueNode};
pub use crate::error::{CoreError, LexError, ParseError, RuntimeError};
pub use crate::lexer::{Token, TokenKind};
pub use crate::parser::{ParseReport, Parser};
pub use crate::runtime::Evaluator;
pub use crate::value::{OrbitNumber, OrbitValue};

pub fn parse(source: &str) -> Result<AstNode, CoreError> {
    let parser = Parser::from_source(source)?;
    let ast = parser.parse_document()?;
    Ok(ast)
}

pub fn evaluate(source: &str) -> Result<OrbitValue, CoreError> {
    let ast = parse(source)?;
    let value = Evaluator::evaluate(&ast)?;
    Ok(value)
}

pub fn evaluate_ast(ast: &AstNode) -> Result<OrbitValue, RuntimeError> {
    Evaluator::evaluate(ast)
}

pub fn parse_with_recovery(source: &str) -> Result<ParseReport, CoreError> {
    let parser = Parser::from_source(source)?;
    Ok(parser.parse_document_with_recovery())
}
