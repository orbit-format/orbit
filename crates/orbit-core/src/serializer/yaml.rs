use crate::value::OrbitValue;

pub fn to_yaml_string(value: &OrbitValue) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(value)
}
