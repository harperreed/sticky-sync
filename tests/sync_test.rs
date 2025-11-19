use sticky_situation::sync::{SyncEngine, SyncAction};
use std::collections::HashMap;

#[test]
fn test_categorize_new_on_filesystem() {
    let fs_uuids = vec!["uuid-1".to_string(), "uuid-2".to_string()];
    let mut db_times = HashMap::new();
    db_times.insert("uuid-2".to_string(), 1000);

    let fs_times = HashMap::from([
        ("uuid-1".to_string(), 2000),
        ("uuid-2".to_string(), 2000),
    ]);

    let actions = SyncEngine::categorize(&fs_uuids, &db_times, &fs_times);

    assert!(actions.iter().any(|a| matches!(a, SyncAction::NewOnFilesystem(uuid) if uuid == "uuid-1")));
}

#[test]
fn test_categorize_modified_last_write_wins() {
    let fs_uuids = vec!["uuid-1".to_string()];
    let db_times = HashMap::from([("uuid-1".to_string(), 1000)]);
    let fs_times = HashMap::from([("uuid-1".to_string(), 2000)]);

    let actions = SyncEngine::categorize(&fs_uuids, &db_times, &fs_times);

    assert!(actions.iter().any(|a| matches!(a, SyncAction::UpdateDatabase(uuid) if uuid == "uuid-1")));
}

#[test]
fn test_categorize_db_newer() {
    let fs_uuids = vec!["uuid-1".to_string()];
    let db_times = HashMap::from([("uuid-1".to_string(), 3000)]);
    let fs_times = HashMap::from([("uuid-1".to_string(), 2000)]);

    let actions = SyncEngine::categorize(&fs_uuids, &db_times, &fs_times);

    assert!(actions.iter().any(|a| matches!(a, SyncAction::UpdateFilesystem(uuid) if uuid == "uuid-1")));
}

#[test]
fn test_categorize_no_change() {
    let fs_uuids = vec!["uuid-1".to_string()];
    let db_times = HashMap::from([("uuid-1".to_string(), 2000)]);
    let fs_times = HashMap::from([("uuid-1".to_string(), 2000)]);

    let actions = SyncEngine::categorize(&fs_uuids, &db_times, &fs_times);

    assert!(actions.iter().any(|a| matches!(a, SyncAction::NoChange(uuid) if uuid == "uuid-1")));
}

#[test]
fn test_categorize_new_in_database() {
    let fs_uuids = vec!["uuid-1".to_string()];
    let db_times = HashMap::from([
        ("uuid-1".to_string(), 2000),
        ("uuid-2".to_string(), 3000),
    ]);
    let fs_times = HashMap::from([("uuid-1".to_string(), 2000)]);

    let actions = SyncEngine::categorize(&fs_uuids, &db_times, &fs_times);

    assert!(actions.iter().any(|a| matches!(a, SyncAction::NewInDatabase(uuid) if uuid == "uuid-2")));
}

#[test]
fn test_categorize_mixed_actions() {
    // Complex scenario with multiple actions
    let fs_uuids = vec![
        "uuid-1".to_string(),  // New on filesystem
        "uuid-2".to_string(),  // FS newer
        "uuid-3".to_string(),  // DB newer
        "uuid-4".to_string(),  // No change
    ];

    let db_times = HashMap::from([
        ("uuid-2".to_string(), 1000),
        ("uuid-3".to_string(), 3000),
        ("uuid-4".to_string(), 2000),
        ("uuid-5".to_string(), 4000),  // Only in DB
    ]);

    let fs_times = HashMap::from([
        ("uuid-1".to_string(), 1500),
        ("uuid-2".to_string(), 2000),
        ("uuid-3".to_string(), 2000),
        ("uuid-4".to_string(), 2000),
    ]);

    let actions = SyncEngine::categorize(&fs_uuids, &db_times, &fs_times);

    // Verify all expected actions are present
    assert!(actions.iter().any(|a| matches!(a, SyncAction::NewOnFilesystem(uuid) if uuid == "uuid-1")));
    assert!(actions.iter().any(|a| matches!(a, SyncAction::UpdateDatabase(uuid) if uuid == "uuid-2")));
    assert!(actions.iter().any(|a| matches!(a, SyncAction::UpdateFilesystem(uuid) if uuid == "uuid-3")));
    assert!(actions.iter().any(|a| matches!(a, SyncAction::NoChange(uuid) if uuid == "uuid-4")));
    assert!(actions.iter().any(|a| matches!(a, SyncAction::NewInDatabase(uuid) if uuid == "uuid-5")));

    // Verify we have exactly 5 actions
    assert_eq!(actions.len(), 5);
}
