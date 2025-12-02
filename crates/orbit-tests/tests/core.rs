use indexmap::IndexMap;
use orbit_core::{
    OrbitNumber, OrbitValue,
    serializer::{to_json_string_pretty, to_yaml_string},
};

const SAMPLE: &str = r#"
server {
    host: "127.0.0.1"
    port: 8080
    tags: [
        "edge",
        "prod"
    ]
}
"#;

#[test]
fn evaluates_sample_document() {
    let value = orbit_core::evaluate(SAMPLE).expect("evaluation should succeed");
    let mut expected = IndexMap::new();
    let mut server = IndexMap::new();
    server.insert("host".into(), OrbitValue::String("127.0.0.1".into()));
    server.insert(
        "port".into(),
        OrbitValue::Number(OrbitNumber::Integer(8080)),
    );
    server.insert(
        "tags".into(),
        OrbitValue::List(vec![
            OrbitValue::String("edge".into()),
            OrbitValue::String("prod".into()),
        ]),
    );
    expected.insert("server".into(), OrbitValue::Object(server));
    assert_eq!(value, OrbitValue::Object(expected));
}

#[test]
fn formatter_is_idempotent() {
    let formatted = orbit_fmt::format_source(SAMPLE).expect("formatting should succeed");
    let formatted_again = orbit_fmt::format_source(&formatted).expect("formatting should succeed");
    assert_eq!(formatted, formatted_again);
}

#[test]
fn serializers_emit_output() {
    let value = orbit_core::evaluate(SAMPLE).expect("evaluation should succeed");
    let json = to_json_string_pretty(&value).expect("json serialization");
    assert!(json.contains("\"server\""));
    let yaml = to_yaml_string(&value).expect("yaml serialization");
    assert!(yaml.contains("server:"));
}
