use crate::ast::{AstNode, Span};

pub type Document = AstNode;

pub fn document(body: Vec<AstNode>, span: Span) -> AstNode {
    AstNode::Document { body, span }
}
