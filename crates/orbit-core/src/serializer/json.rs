use crate::value::OrbitValue;

pub fn to_json_string(value: &OrbitValue) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub fn to_json_string_pretty(value: &OrbitValue) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}
