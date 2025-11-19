use sticky_situation::database::Database;
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
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    assert!(tables.contains(&"stickies".to_string()));
    assert!(tables.contains(&"attachments".to_string()));
}
