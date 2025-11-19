use sticky_situation::filesystem::rtfd::RtfdBundle;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_read_rtfd_bundle() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("test.rtfd");
    fs::create_dir(&rtfd_dir).unwrap();

    let rtf_content = r"{\rtf1\ansi\ansicpg1252 Test content}";
    fs::write(rtfd_dir.join("TXT.rtf"), rtf_content).unwrap();

    let bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    assert_eq!(bundle.rtf_data, rtf_content.as_bytes());
    assert!(bundle.attachments.is_empty());
}

#[test]
fn test_read_rtfd_with_attachments() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("test.rtfd");
    fs::create_dir(&rtfd_dir).unwrap();

    fs::write(rtfd_dir.join("TXT.rtf"), "test").unwrap();
    fs::write(rtfd_dir.join("IMG_001.tiff"), b"fake image").unwrap();

    let bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    assert_eq!(bundle.attachments.len(), 1);
    assert_eq!(bundle.attachments[0].filename, "IMG_001.tiff");
}
