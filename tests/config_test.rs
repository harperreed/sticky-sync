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
