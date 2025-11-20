use std::fs;
use std::path::Path;
use sticky_situation::database::{Database, Sticky};
use tempfile::tempdir;

#[test]
fn test_create_database() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let _db = Database::create(&db_path).unwrap();
    assert!(db_path.exists());
}

#[test]
fn test_schema_created() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let db = Database::create(&db_path).unwrap();

    // Verify tables exist
    let conn = db.connection().borrow();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap();
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    assert!(tables.contains(&"stickies".to_string()));
    assert!(tables.contains(&"attachments".to_string()));
}

#[test]
fn test_database_creation_invalid_path() {
    // Try to create database in a path that doesn't exist and can't be created
    let result = Database::create(Path::new("/nonexistent/deeply/nested/path/test.db"));
    assert!(result.is_err());
}

#[test]
fn test_search_with_special_characters() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::create(&db_path).unwrap();

    // Insert sticky with special characters
    let sticky = Sticky {
        uuid: "test-123".to_string(),
        content_text: "Test with special chars: @#$% & * () []".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };
    db.insert_sticky(&sticky).unwrap();

    // Search for content with special characters
    let results = db.search("special").unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_search_with_quotes() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::create(&db_path).unwrap();

    // Insert sticky with quotes
    let sticky = Sticky {
        uuid: "test-123".to_string(),
        content_text: r#"This has "quoted text" in it"#.to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };
    db.insert_sticky(&sticky).unwrap();

    // Search should work even with quotes in content
    let results = db.search("quoted").unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_database_insert_and_retrieve() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::create(&db_path).unwrap();

    let sticky = Sticky {
        uuid: "abc-123".to_string(),
        content_text: "Test content".to_string(),
        rtf_data: vec![1, 2, 3],
        plist_metadata: vec![],
        color: "blue".to_string(),
        modified_at: 5000,
        created_at: 1000,
        source_machine: "machine1".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    let retrieved = db.get_sticky("abc-123").unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.uuid, "abc-123");
    assert_eq!(retrieved.content_text, "Test content");
    assert_eq!(retrieved.color, "blue");
}

#[test]
fn test_database_update_sticky() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::create(&db_path).unwrap();

    // Insert original
    let sticky = Sticky {
        uuid: "update-test".to_string(),
        content_text: "Original content".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };
    db.insert_sticky(&sticky).unwrap();

    // Update with new content
    let updated = Sticky {
        uuid: "update-test".to_string(),
        content_text: "Updated content".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "blue".to_string(),
        modified_at: 2000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };
    db.insert_sticky(&updated).unwrap();

    // Verify update
    let retrieved = db.get_sticky("update-test").unwrap().unwrap();
    assert_eq!(retrieved.content_text, "Updated content");
    assert_eq!(retrieved.color, "blue");
    assert_eq!(retrieved.modified_at, 2000);
}

#[test]
fn test_search_empty_database() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::create(&db_path).unwrap();

    let results = db.search("anything").unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_get_nonexistent_sticky() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::create(&db_path).unwrap();

    let result = db.get_sticky("does-not-exist").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_database_with_readonly_path() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Create database first
    {
        let _db = Database::create(&db_path).unwrap();
    }

    // Make it readonly
    let mut perms = fs::metadata(&db_path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&db_path, perms).unwrap();

    // Try to open readonly database - this should still work for reads
    let result = Database::create(&db_path);

    // Clean up - remove readonly before temp dir cleanup
    let mut perms = fs::metadata(&db_path).unwrap().permissions();
    perms.set_readonly(false);
    fs::set_permissions(&db_path, perms).unwrap();

    // Database should open successfully (SQLite can open readonly)
    assert!(result.is_ok());
}
