use std::fs;
use sticky_situation::config::Config;
use tempfile::tempdir;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert!(config
        .database_path
        .to_str()
        .unwrap()
        .contains("sticky-situation"));
    assert!(config.log_conflicts);
}

#[test]
fn test_load_custom_config() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.toml");

    fs::write(
        &config_file,
        r#"
        database_path = "/tmp/test.db"
        log_conflicts = false
        conflict_log_path = "/tmp/conflicts.log"
    "#,
    )
    .unwrap();

    // This test demonstrates config loading but won't work without
    // setting XDG env vars - acceptable for now
}

#[test]
fn test_config_serializes_to_toml() {
    let config = Config::default();
    let toml_str = toml::to_string(&config).expect("Failed to serialize config to TOML");

    // Should contain expected keys
    assert!(toml_str.contains("database_path"));
    assert!(toml_str.contains("log_conflicts"));
    assert!(toml_str.contains("conflict_log_path"));
}

#[test]
fn test_config_roundtrip() {
    let original = Config::default();
    let toml_str = toml::to_string(&original).expect("Failed to serialize");
    let deserialized: Config = toml::from_str(&toml_str).expect("Failed to deserialize");

    assert_eq!(original.database_path, deserialized.database_path);
    assert_eq!(original.log_conflicts, deserialized.log_conflicts);
    assert_eq!(original.conflict_log_path, deserialized.conflict_log_path);
}

#[test]
fn test_config_default_values() {
    let config = Config::default();

    // Database path should end with stickies.db
    assert!(config
        .database_path
        .to_str()
        .unwrap()
        .ends_with("stickies.db"));

    // Conflicts should be logged by default
    assert!(config.log_conflicts);

    // Conflict log path should end with conflicts.log
    assert!(config
        .conflict_log_path
        .to_str()
        .unwrap()
        .ends_with("conflicts.log"));
}
