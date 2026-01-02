use orchestrator::config::OrchestratorConfig;

#[test]
fn valid_config_loads() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/orchestrator.json", manifest);
    let s = std::fs::read_to_string(path).unwrap();
    let cfg = OrchestratorConfig::load_from_str(&s);
    assert!(cfg.is_ok());
}

#[test]
fn missing_archetype_fails() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/orchestrator.json", manifest);
    let mut s = std::fs::read_to_string(path).unwrap();
    s = s.replace("\n    \"Jester\": {", "\n    \"Jester_MISSING\": {");
    let cfg = OrchestratorConfig::load_from_str(&s);
    assert!(cfg.is_err());
}

#[test]
fn extra_fields_fail() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/orchestrator.json", manifest);
    let mut s = std::fs::read_to_string(path).unwrap();
    s = s.replacen("}", ",\n  \"extra\": 1\n}", 1);
    let cfg = OrchestratorConfig::load_from_str(&s);
    assert!(cfg.is_err());
}

#[test]
fn sentinel_temp_must_be_zero() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/orchestrator.json", manifest);
    let mut s = std::fs::read_to_string(path).unwrap();
    s = s.replace("\"temperature\": 0.0", "\"temperature\": 0.1");
    let cfg = OrchestratorConfig::load_from_str(&s);
    assert!(cfg.is_err());
}

#[test]
fn invalid_temp_range_fails() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/orchestrator.json", manifest);
    let mut s = std::fs::read_to_string(path).unwrap();
    s = s.replace("\"temperature\": 0.8", "\"temperature\": 1.5");
    let cfg = OrchestratorConfig::load_from_str(&s);
    assert!(cfg.is_err());
}
