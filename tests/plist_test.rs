use plist::{Dictionary, Value};
use sticky_situation::filesystem::plist::StickyMetadata;

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
}
