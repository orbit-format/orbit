use indexmap::IndexMap;

use crate::{
    ast::{AstNode, ObjectEntry, ValueNode},
    error::RuntimeError,
    value::OrbitValue,
};

use super::environment::Environment;

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(ast: &AstNode) -> Result<OrbitValue, RuntimeError> {
        match ast {
            AstNode::Document { body, .. } => Self::evaluate_nodes(body),
            node => Self::evaluate_nodes(std::slice::from_ref(node)),
        }
    }

    fn evaluate_nodes(nodes: &[AstNode]) -> Result<OrbitValue, RuntimeError> {
        let mut env = Environment::new();
        for node in nodes {
            match node {
                AstNode::Entry { key, value, span } => {
                    let evaluated = Self::evaluate_value(value)?;
                    if env.insert(key.clone(), evaluated).is_some() {
                        return Err(RuntimeError::new(format!("duplicate key '{key}'"), *span));
                    }
                }
                AstNode::Block { name, body, span } => {
                    let nested = Self::evaluate_nodes(body)?;
                    if env.insert(name.clone(), nested).is_some() {
                        return Err(RuntimeError::new(
                            format!("duplicate block '{name}'"),
                            *span,
                        ));
                    }
                }
                AstNode::Document { body, .. } => {
                    let nested = Self::evaluate_nodes(body)?;
                    if let OrbitValue::Object(map) = nested {
                        for (key, value) in map {
                            if env.insert(key.clone(), value).is_some() {
                                return Err(RuntimeError::new(
                                    format!("duplicate key '{key}'"),
                                    node.span(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(env.into_value())
    }

    fn evaluate_value(value: &ValueNode) -> Result<OrbitValue, RuntimeError> {
        match value {
            ValueNode::String { value, .. } => Ok(OrbitValue::String(value.clone())),
            ValueNode::Number { value, .. } => Ok(OrbitValue::Number(*value)),
            ValueNode::Bool { value, .. } => Ok(OrbitValue::Bool(*value)),
            ValueNode::List { items, .. } => {
                let mut evaluated = Vec::with_capacity(items.len());
                for item in items {
                    evaluated.push(Self::evaluate_value(item)?);
                }
                Ok(OrbitValue::List(evaluated))
            }
            ValueNode::Object { entries, .. } => {
                let map = Self::evaluate_object_entries(entries)?;
                Ok(OrbitValue::Object(map))
            }
        }
    }

    fn evaluate_object_entries(
        entries: &[ObjectEntry],
    ) -> Result<IndexMap<String, OrbitValue>, RuntimeError> {
        let mut map = IndexMap::new();
        for entry in entries {
            let value = Self::evaluate_value(&entry.value)?;
            if map.insert(entry.key.clone(), value).is_some() {
                return Err(RuntimeError::new(
                    format!("duplicate key '{}' inside object literal", entry.key),
                    entry.span,
                ));
            }
        }
        Ok(map)
    }
}
