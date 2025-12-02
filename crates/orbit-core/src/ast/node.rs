use crate::value::number::OrbitNumber;
use serde::{Deserialize, Serialize};

use super::span::Span;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AstNode {
    Document {
        body: Vec<AstNode>,
        span: Span,
    },
    Entry {
        key: String,
        value: ValueNode,
        span: Span,
    },
    Block {
        name: String,
        body: Vec<AstNode>,
        span: Span,
    },
}

impl AstNode {
    pub fn span(&self) -> Span {
        match self {
            AstNode::Document { span, .. }
            | AstNode::Entry { span, .. }
            | AstNode::Block { span, .. } => *span,
        }
    }

    pub fn as_body(&self) -> Option<&[AstNode]> {
        match self {
            AstNode::Document { body, .. } | AstNode::Block { body, .. } => Some(body),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ValueNode {
    String {
        value: String,
        span: Span,
    },
    Number {
        value: OrbitNumber,
        span: Span,
    },
    Bool {
        value: bool,
        span: Span,
    },
    List {
        items: Vec<ValueNode>,
        span: Span,
    },
    Object {
        entries: Vec<ObjectEntry>,
        span: Span,
    },
}

impl ValueNode {
    pub fn span(&self) -> Span {
        match self {
            ValueNode::String { span, .. }
            | ValueNode::Number { span, .. }
            | ValueNode::Bool { span, .. }
            | ValueNode::List { span, .. }
            | ValueNode::Object { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectEntry {
    pub key: String,
    pub value: ValueNode,
    pub span: Span,
}
