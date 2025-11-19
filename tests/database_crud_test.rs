use sticky_situation::database::{Database, Sticky};
use tempfile::tempdir;

#[test]
fn test_insert_sticky() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    let sticky = Sticky {
        uuid: "test-uuid-123".to_string(),
        content_text: "Hello world".to_string(),
        rtf_data: b"rtf data".to_vec(),
        plist_metadata: b"plist data".to_vec(),
        color: "yellow".to_string(),
        modified_at: 1234567890,
        created_at: 1234567890,
        source_machine: "test-machine".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    let loaded = db.get_sticky("test-uuid-123").unwrap().unwrap();
    assert_eq!(loaded.content_text, "Hello world");
}

#[test]
fn test_search_stickies() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    let sticky = Sticky {
        uuid: "test-uuid-456".to_string(),
        content_text: "Meeting notes for project".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "blue".to_string(),
        modified_at: 1234567890,
        created_at: 1234567890,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    let results = db.search("meeting").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].uuid, "test-uuid-456");
}

#[test]
fn test_get_sticky_not_found() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    let result = db.get_sticky("nonexistent-uuid").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_all_uuids() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    let sticky1 = Sticky {
        uuid: "uuid-1".to_string(),
        content_text: "First sticky".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    let sticky2 = Sticky {
        uuid: "uuid-2".to_string(),
        content_text: "Second sticky".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "blue".to_string(),
        modified_at: 2000,
        created_at: 2000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky1).unwrap();
    db.insert_sticky(&sticky2).unwrap();

    let uuids = db.get_all_uuids().unwrap();
    assert_eq!(uuids.len(), 2);
    assert!(uuids.contains(&"uuid-1".to_string()));
    assert!(uuids.contains(&"uuid-2".to_string()));
}

#[test]
fn test_insert_or_replace() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    // Insert initial sticky
    let sticky1 = Sticky {
        uuid: "same-uuid".to_string(),
        content_text: "Original content".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky1).unwrap();

    // Replace with new content
    let sticky2 = Sticky {
        uuid: "same-uuid".to_string(),
        content_text: "Updated content".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "blue".to_string(),
        modified_at: 2000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky2).unwrap();

    // Verify only one sticky exists and it has updated content
    let uuids = db.get_all_uuids().unwrap();
    assert_eq!(uuids.len(), 1);

    let loaded = db.get_sticky("same-uuid").unwrap().unwrap();
    assert_eq!(loaded.content_text, "Updated content");
    assert_eq!(loaded.color, "blue");
    assert_eq!(loaded.modified_at, 2000);
}

#[test]
fn test_fts5_index_sync() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    // Insert sticky
    let sticky = Sticky {
        uuid: "fts-test".to_string(),
        content_text: "This has searchable keywords".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    // Search should find it
    let results = db.search("searchable").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].uuid, "fts-test");

    // Update the content
    let updated = Sticky {
        uuid: "fts-test".to_string(),
        content_text: "This has different words now".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 2000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&updated).unwrap();

    // Old keyword should not be found
    let old_results = db.search("searchable").unwrap();
    assert_eq!(old_results.len(), 0);

    // New keyword should be found
    let new_results = db.search("different").unwrap();
    assert_eq!(new_results.len(), 1);
    assert_eq!(new_results[0].uuid, "fts-test");
}

#[test]
fn test_search_multiple_results() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    let sticky1 = Sticky {
        uuid: "search-1".to_string(),
        content_text: "Important meeting tomorrow".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    let sticky2 = Sticky {
        uuid: "search-2".to_string(),
        content_text: "Meeting notes from last week".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "blue".to_string(),
        modified_at: 2000,
        created_at: 2000,
        source_machine: "test".to_string(),
    };

    let sticky3 = Sticky {
        uuid: "search-3".to_string(),
        content_text: "Grocery list: milk, eggs, bread".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "green".to_string(),
        modified_at: 3000,
        created_at: 3000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky1).unwrap();
    db.insert_sticky(&sticky2).unwrap();
    db.insert_sticky(&sticky3).unwrap();

    let results = db.search("meeting").unwrap();
    assert_eq!(results.len(), 2);

    let uuids: Vec<String> = results.iter().map(|s| s.uuid.clone()).collect();
    assert!(uuids.contains(&"search-1".to_string()));
    assert!(uuids.contains(&"search-2".to_string()));
}

#[test]
fn test_search_no_results() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    let sticky = Sticky {
        uuid: "only-sticky".to_string(),
        content_text: "Nothing to see here".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    let results = db.search("nonexistent").unwrap();
    assert_eq!(results.len(), 0);
}
