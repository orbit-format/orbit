use indexmap::IndexMap;

use crate::value::OrbitValue;

#[derive(Debug, Default, Clone)]
pub struct Environment {
    values: IndexMap<String, OrbitValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: OrbitValue) -> Option<OrbitValue> {
        self.values.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&OrbitValue> {
        self.values.get(key)
    }

    pub fn into_value(self) -> OrbitValue {
        OrbitValue::Object(self.values)
    }
}
