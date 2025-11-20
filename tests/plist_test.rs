use plist::{Dictionary, Value};
use sticky_situation::filesystem::plist::{read_stickies_state, StickyMetadata};
use tempfile::tempdir;

#[test]
fn test_parse_sticky_metadata() {
    let mut dict = Dictionary::new();
    dict.insert("Color".to_string(), Value::Integer(0.into()));
    dict.insert(
        "Frame".to_string(),
        Value::String("{{100, 200}, {300, 400}}".into()),
    );

    let metadata = StickyMetadata::from_plist_dict(&dict).unwrap();
    assert_eq!(metadata.color_index, 0);
    assert_eq!(metadata.frame, "{{100, 200}, {300, 400}}");
}

#[test]
fn test_parse_metadata_with_missing_color() {
    let mut dict = Dictionary::new();
    dict.insert(
        "Frame".to_string(),
        Value::String("{{100, 200}, {300, 400}}".into()),
    );
    dict.insert("Floating".to_string(), Value::Boolean(false));

    // Should use default color (0 = yellow)
    let metadata = StickyMetadata::from_plist_dict(&dict).unwrap();
    assert_eq!(metadata.color_index, 0);
    assert_eq!(metadata.color_name(), "yellow");
}

#[test]
fn test_parse_metadata_with_missing_frame() {
    let mut dict = Dictionary::new();
    dict.insert("Color".to_string(), Value::Integer(1.into()));
    dict.insert("Floating".to_string(), Value::Boolean(true));

    // Should use default frame
    let metadata = StickyMetadata::from_plist_dict(&dict).unwrap();
    assert_eq!(metadata.frame, "{{100, 100}, {250, 250}}");
    assert!(metadata.is_floating);
}

#[test]
fn test_parse_metadata_with_missing_floating() {
    let mut dict = Dictionary::new();
    dict.insert("Color".to_string(), Value::Integer(2.into()));
    dict.insert(
        "Frame".to_string(),
        Value::String("{{50, 50}, {200, 200}}".into()),
    );

    // Should default to false
    let metadata = StickyMetadata::from_plist_dict(&dict).unwrap();
    assert!(!metadata.is_floating);
}

#[test]
fn test_color_name_mapping() {
    let colors = vec![
        (0, "yellow"),
        (1, "blue"),
        (2, "green"),
        (3, "pink"),
        (4, "purple"),
        (5, "gray"),
        (99, "yellow"), // Unknown defaults to yellow
    ];

    for (index, expected_name) in colors {
        let mut dict = Dictionary::new();
        dict.insert("Color".to_string(), Value::Integer(index.into()));
        dict.insert(
            "Frame".to_string(),
            Value::String("{{0, 0}, {100, 100}}".into()),
        );

        let metadata = StickyMetadata::from_plist_dict(&dict).unwrap();
        assert_eq!(metadata.color_name(), expected_name);
    }
}

#[test]
fn test_read_dictionary_format_plist() {
    let dir = tempdir().unwrap();
    let plist_path = dir.path().join("test.plist");

    // Create a dictionary-format plist (old StickiesState.plist format)
    let mut root = Dictionary::new();

    let mut sticky1 = Dictionary::new();
    sticky1.insert("Color".to_string(), Value::Integer(0.into()));
    sticky1.insert(
        "Frame".to_string(),
        Value::String("{{100, 100}, {250, 250}}".into()),
    );
    sticky1.insert("Floating".to_string(), Value::Boolean(false));

    root.insert("abc-123-def".to_string(), Value::Dictionary(sticky1));

    let value = Value::Dictionary(root);
    plist::to_file_xml(&plist_path, &value).unwrap();

    let result = read_stickies_state(&plist_path).unwrap();
    assert_eq!(result.len(), 1);
    assert!(result.contains_key("abc-123-def"));
}

#[test]
fn test_read_array_format_plist() {
    let dir = tempdir().unwrap();
    let plist_path = dir.path().join("test.plist");

    // Create an array-format plist (.SavedStickiesState format)
    let mut entry = Dictionary::new();
    entry.insert("UUID".to_string(), Value::String("ABC-123-DEF".into()));
    entry.insert("Color".to_string(), Value::Integer(1.into()));
    entry.insert(
        "Frame".to_string(),
        Value::String("{{200, 200}, {300, 300}}".into()),
    );
    entry.insert("Floating".to_string(), Value::Boolean(true));

    let array = vec![Value::Dictionary(entry)];
    let value = Value::Array(array);
    plist::to_file_xml(&plist_path, &value).unwrap();

    let result = read_stickies_state(&plist_path).unwrap();
    assert_eq!(result.len(), 1);
    // UUID should be lowercased
    assert!(result.contains_key("abc-123-def"));

    let metadata = result.get("abc-123-def").unwrap();
    assert_eq!(metadata.color_index, 1);
    assert_eq!(metadata.color_name(), "blue");
    assert!(metadata.is_floating);
}

#[test]
fn test_read_nonexistent_plist() {
    let dir = tempdir().unwrap();
    let plist_path = dir.path().join("nonexistent.plist");

    // Should return empty HashMap, not error
    let result = read_stickies_state(&plist_path).unwrap();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_uuid_case_handling() {
    let dir = tempdir().unwrap();
    let plist_path = dir.path().join("test.plist");

    // Create array with mixed-case UUID
    let mut entry = Dictionary::new();
    entry.insert("UUID".to_string(), Value::String("AbC-123-DeF".into()));
    entry.insert("Color".to_string(), Value::Integer(0.into()));
    entry.insert(
        "Frame".to_string(),
        Value::String("{{0, 0}, {100, 100}}".into()),
    );

    let array = vec![Value::Dictionary(entry)];
    plist::to_file_xml(&plist_path, &Value::Array(array)).unwrap();

    let result = read_stickies_state(&plist_path).unwrap();

    // Should be normalized to lowercase
    assert!(result.contains_key("abc-123-def"));
    assert!(!result.contains_key("AbC-123-DeF"));
}

#[test]
fn test_read_multiple_entries_array_format() {
    let dir = tempdir().unwrap();
    let plist_path = dir.path().join("test.plist");

    let mut entries = Vec::new();

    for i in 0..5 {
        let mut entry = Dictionary::new();
        entry.insert("UUID".to_string(), Value::String(format!("uuid-{}", i)));
        entry.insert("Color".to_string(), Value::Integer(i.into()));
        entry.insert(
            "Frame".to_string(),
            Value::String(format!("{{{{{}, {}}}, {{100, 100}}}}", i * 10, i * 10)),
        );
        entries.push(Value::Dictionary(entry));
    }

    plist::to_file_xml(&plist_path, &Value::Array(entries)).unwrap();

    let result = read_stickies_state(&plist_path).unwrap();
    assert_eq!(result.len(), 5);

    for i in 0..5 {
        let uuid = format!("uuid-{}", i);
        assert!(result.contains_key(&uuid));
        let metadata = result.get(&uuid).unwrap();
        assert_eq!(metadata.color_index, i);
    }
}

#[test]
fn test_read_empty_array_plist() {
    let dir = tempdir().unwrap();
    let plist_path = dir.path().join("test.plist");

    let empty_array: Vec<Value> = vec![];
    plist::to_file_xml(&plist_path, &Value::Array(empty_array)).unwrap();

    let result = read_stickies_state(&plist_path).unwrap();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_read_empty_dictionary_plist() {
    let dir = tempdir().unwrap();
    let plist_path = dir.path().join("test.plist");

    let empty_dict = Dictionary::new();
    plist::to_file_xml(&plist_path, &Value::Dictionary(empty_dict)).unwrap();

    let result = read_stickies_state(&plist_path).unwrap();
    assert_eq!(result.len(), 0);
}
