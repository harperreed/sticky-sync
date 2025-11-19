use sticky_situation::{
    database::{Database, Sticky},
    filesystem::rtfd::RtfdBundle,
    rtf,
};
use tempfile::tempdir;

#[test]
fn test_full_sync_workflow() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Create database
    let _db = Database::create(&db_path).unwrap();

    // Create fake RTFD bundle
    let rtfd_dir = dir.path().join("test-uuid.rtfd");
    let bundle = RtfdBundle::create_minimal("Integration test content");
    bundle.write(&rtfd_dir).unwrap();

    // Read it back
    let read_bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    assert!(!read_bundle.rtf_data.is_empty());
}

#[test]
fn test_search_integration() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    // Insert test sticky
    let sticky = Sticky {
        uuid: "search-test".to_string(),
        content_text: "Find this unique phrase".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    // Search for it
    let results = db.search("unique").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].uuid, "search-test");
}

#[test]
fn test_rtfd_to_database_to_search() {
    // End-to-end: RTFD reading → database storage → search
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::create(&db_path).unwrap();

    // Step 1: Create RTFD bundle with rich content
    let rtfd_dir = dir.path().join("uuid-123.rtfd");
    let test_content = "Meeting notes about the quarterly review";
    let bundle = RtfdBundle::create_minimal(test_content);
    bundle.write(&rtfd_dir).unwrap();

    // Step 2: Read RTFD and extract text
    let read_bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    let extracted_text = rtf::extract_text_from_bytes(&read_bundle.rtf_data);

    // Verify text extraction works
    assert!(extracted_text.contains("Meeting notes"));
    assert!(extracted_text.contains("quarterly review"));

    // Step 3: Store in database
    let sticky = Sticky {
        uuid: "uuid-123".to_string(),
        content_text: extracted_text.clone(),
        rtf_data: read_bundle.rtf_data.clone(),
        plist_metadata: vec![],
        color: "blue".to_string(),
        modified_at: 1234567890,
        created_at: 1234567890,
        source_machine: "integration-test".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    // Step 4: Search for content
    let results = db.search("quarterly").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].uuid, "uuid-123");
    assert_eq!(results[0].color, "blue");

    // Verify full content is preserved
    let retrieved = db.get_sticky("uuid-123").unwrap().unwrap();
    assert_eq!(retrieved.content_text, extracted_text);
    assert_eq!(retrieved.rtf_data, read_bundle.rtf_data);
}

#[test]
fn test_multiple_stickies_search() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    // Create multiple stickies
    let stickies = vec![
        Sticky {
            uuid: "sticky-1".to_string(),
            content_text: "Buy groceries: milk, eggs, bread".to_string(),
            rtf_data: vec![],
            plist_metadata: vec![],
            color: "yellow".to_string(),
            modified_at: 1000,
            created_at: 1000,
            source_machine: "test".to_string(),
        },
        Sticky {
            uuid: "sticky-2".to_string(),
            content_text: "Meeting at 3pm tomorrow".to_string(),
            rtf_data: vec![],
            plist_metadata: vec![],
            color: "blue".to_string(),
            modified_at: 2000,
            created_at: 2000,
            source_machine: "test".to_string(),
        },
        Sticky {
            uuid: "sticky-3".to_string(),
            content_text: "Call mom about her birthday".to_string(),
            rtf_data: vec![],
            plist_metadata: vec![],
            color: "pink".to_string(),
            modified_at: 3000,
            created_at: 3000,
            source_machine: "test".to_string(),
        },
    ];

    for sticky in &stickies {
        db.insert_sticky(sticky).unwrap();
    }

    // Search for specific terms
    let grocery_results = db.search("groceries").unwrap();
    assert_eq!(grocery_results.len(), 1);
    assert_eq!(grocery_results[0].uuid, "sticky-1");

    let meeting_results = db.search("meeting").unwrap();
    assert_eq!(meeting_results.len(), 1);
    assert_eq!(meeting_results[0].uuid, "sticky-2");

    // Search for term that doesn't exist
    let no_results = db.search("nonexistent").unwrap();
    assert_eq!(no_results.len(), 0);
}

#[test]
fn test_rtfd_with_attachments_workflow() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::create(&db_path).unwrap();

    // Create RTFD with attachment
    let rtfd_dir = dir.path().join("with-attachment.rtfd");
    let bundle = RtfdBundle {
        rtf_data: b"{\\rtf1\\ansi Test with image}".to_vec(),
        attachments: vec![
            sticky_situation::filesystem::rtfd::Attachment {
                filename: "image.png".to_string(),
                content: b"fake image data".to_vec(),
            },
        ],
    };
    bundle.write(&rtfd_dir).unwrap();

    // Read it back
    let read_bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    assert_eq!(read_bundle.attachments.len(), 1);
    assert_eq!(read_bundle.attachments[0].filename, "image.png");

    // Store in database
    let extracted_text = rtf::extract_text_from_bytes(&read_bundle.rtf_data);
    let sticky = Sticky {
        uuid: "with-attachment".to_string(),
        content_text: extracted_text,
        rtf_data: read_bundle.rtf_data,
        plist_metadata: vec![],
        color: "green".to_string(),
        modified_at: 5000,
        created_at: 5000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    // Search and verify
    let results = db.search("image").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].uuid, "with-attachment");
}

#[test]
fn test_update_sticky_preserves_search() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    // Insert initial sticky
    let sticky_v1 = Sticky {
        uuid: "updatable".to_string(),
        content_text: "Original content".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky_v1).unwrap();

    // Verify it's searchable
    let results = db.search("Original").unwrap();
    assert_eq!(results.len(), 1);

    // Update the sticky
    let sticky_v2 = Sticky {
        uuid: "updatable".to_string(),
        content_text: "Updated content with new information".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "blue".to_string(),
        modified_at: 2000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky_v2).unwrap();

    // Old search term should not find it
    let old_results = db.search("Original").unwrap();
    assert_eq!(old_results.len(), 0);

    // New search term should find it
    let new_results = db.search("information").unwrap();
    assert_eq!(new_results.len(), 1);
    assert_eq!(new_results[0].content_text, "Updated content with new information");
    assert_eq!(new_results[0].color, "blue");
}

#[test]
fn test_rtfd_roundtrip_preserves_content() {
    let dir = tempdir().unwrap();

    // Create original RTFD
    let original_text = "Important todo list";
    let original_bundle = RtfdBundle::create_minimal(original_text);

    // Write to disk
    let rtfd_path = dir.path().join("roundtrip.rtfd");
    original_bundle.write(&rtfd_path).unwrap();

    // Read back
    let read_bundle = RtfdBundle::read(&rtfd_path).unwrap();

    // Extract text and verify
    let extracted = rtf::extract_text_from_bytes(&read_bundle.rtf_data);
    assert!(extracted.contains(original_text));

    // Store in database
    let db_path = dir.path().join("test.db");
    let db = Database::create(&db_path).unwrap();

    let sticky = Sticky {
        uuid: "roundtrip-test".to_string(),
        content_text: extracted,
        rtf_data: read_bundle.rtf_data.clone(),
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    // Retrieve from database
    let retrieved = db.get_sticky("roundtrip-test").unwrap().unwrap();

    // Write back to new RTFD
    let new_rtfd_path = dir.path().join("roundtrip-new.rtfd");
    let new_bundle = RtfdBundle {
        rtf_data: retrieved.rtf_data,
        attachments: vec![],
    };
    new_bundle.write(&new_rtfd_path).unwrap();

    // Verify new RTFD is readable
    let final_bundle = RtfdBundle::read(&new_rtfd_path).unwrap();
    let final_text = rtf::extract_text_from_bytes(&final_bundle.rtf_data);
    assert!(final_text.contains(original_text));
}

#[test]
fn test_database_persistence() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("persistent.db");

    // Create database and insert sticky
    {
        let db = Database::create(&db_path).unwrap();
        let sticky = Sticky {
            uuid: "persistent-1".to_string(),
            content_text: "This should persist".to_string(),
            rtf_data: vec![],
            plist_metadata: vec![],
            color: "yellow".to_string(),
            modified_at: 1000,
            created_at: 1000,
            source_machine: "test".to_string(),
        };
        db.insert_sticky(&sticky).unwrap();
    }

    // Open database again and verify data persists
    {
        let db = Database::create(&db_path).unwrap();
        let retrieved = db.get_sticky("persistent-1").unwrap().unwrap();
        assert_eq!(retrieved.content_text, "This should persist");

        // Search should also work
        let results = db.search("persist").unwrap();
        assert_eq!(results.len(), 1);
    }
}
