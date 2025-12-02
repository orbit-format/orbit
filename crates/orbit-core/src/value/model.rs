use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::number::OrbitNumber;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrbitValue {
    String(String),
    Number(OrbitNumber),
    Bool(bool),
    List(Vec<OrbitValue>),
    Object(IndexMap<String, OrbitValue>),
}

impl OrbitValue {
    pub fn as_object(&self) -> Option<&IndexMap<String, OrbitValue>> {
        match self {
            OrbitValue::Object(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[OrbitValue]> {
        match self {
            OrbitValue::List(values) => Some(values),
            _ => None,
        }
    }

    pub fn get_path<'a>(&'a self, path: &[&str]) -> Option<&'a OrbitValue> {
        let mut current = self;
        for part in path {
            current = match current {
                OrbitValue::Object(map) => map.get(*part)?,
                _ => return None,
            };
        }
        Some(current)
    }
}
