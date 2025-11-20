use std::fs;
use std::os::unix::fs::PermissionsExt;
use sticky_situation::filesystem::rtfd::RtfdBundle;
use tempfile::tempdir;

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

#[test]
fn test_read_rtfd_missing_txt_file() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("test.rtfd");
    fs::create_dir(&rtfd_dir).unwrap();

    // Create RTFD directory without TXT.rtf
    let result = RtfdBundle::read(&rtfd_dir);
    assert!(result.is_err());
}

#[test]
fn test_read_nonexistent_rtfd() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("nonexistent.rtfd");

    let result = RtfdBundle::read(&rtfd_dir);
    assert!(result.is_err());
}

#[test]
fn test_read_rtfd_with_permission_denied() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("test.rtfd");
    fs::create_dir(&rtfd_dir).unwrap();

    let txt_path = rtfd_dir.join("TXT.rtf");
    fs::write(&txt_path, "test content").unwrap();

    // Make file unreadable
    let mut perms = fs::metadata(&txt_path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&txt_path, perms).unwrap();

    let result = RtfdBundle::read(&rtfd_dir);

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&txt_path).unwrap().permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&txt_path, perms).unwrap();

    assert!(result.is_err());
}

#[test]
fn test_read_rtfd_with_multiple_attachments() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("test.rtfd");
    fs::create_dir(&rtfd_dir).unwrap();

    fs::write(rtfd_dir.join("TXT.rtf"), "test").unwrap();
    fs::write(rtfd_dir.join("IMG_001.png"), b"fake png").unwrap();
    fs::write(rtfd_dir.join("IMG_002.jpg"), b"fake jpg").unwrap();
    fs::write(rtfd_dir.join("DOC_001.pdf"), b"fake pdf").unwrap();

    let bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    assert_eq!(bundle.attachments.len(), 3);

    let filenames: Vec<String> = bundle
        .attachments
        .iter()
        .map(|a| a.filename.clone())
        .collect();
    assert!(filenames.contains(&"IMG_001.png".to_string()));
    assert!(filenames.contains(&"IMG_002.jpg".to_string()));
    assert!(filenames.contains(&"DOC_001.pdf".to_string()));
}

#[test]
fn test_read_rtfd_attachment_filenames_with_special_chars() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("test.rtfd");
    fs::create_dir(&rtfd_dir).unwrap();

    fs::write(rtfd_dir.join("TXT.rtf"), "test").unwrap();
    fs::write(rtfd_dir.join("file with spaces.png"), b"data").unwrap();
    fs::write(rtfd_dir.join("file-with-dash.jpg"), b"data").unwrap();
    fs::write(rtfd_dir.join("file_under_score.gif"), b"data").unwrap();

    let bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    assert_eq!(bundle.attachments.len(), 3);
}

#[test]
fn test_rtfd_modified_time() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("test.rtfd");
    fs::create_dir(&rtfd_dir).unwrap();

    fs::write(rtfd_dir.join("TXT.rtf"), "test").unwrap();

    let mtime = RtfdBundle::modified_time(&rtfd_dir).unwrap();
    assert!(mtime > 0);
}

#[test]
fn test_rtfd_modified_time_nonexistent() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("nonexistent.rtfd");

    let result = RtfdBundle::modified_time(&rtfd_dir);
    assert!(result.is_err());
}

#[test]
fn test_read_rtfd_empty_txt_file() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("test.rtfd");
    fs::create_dir(&rtfd_dir).unwrap();

    // Create empty TXT.rtf
    fs::write(rtfd_dir.join("TXT.rtf"), "").unwrap();

    let bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    assert_eq!(bundle.rtf_data.len(), 0);
    assert!(bundle.attachments.is_empty());
}

#[test]
fn test_read_rtfd_large_rtf_content() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("test.rtfd");
    fs::create_dir(&rtfd_dir).unwrap();

    // Create a large RTF file (1MB)
    let large_content = "a".repeat(1024 * 1024);
    fs::write(rtfd_dir.join("TXT.rtf"), &large_content).unwrap();

    let bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    assert_eq!(bundle.rtf_data.len(), large_content.len());
}

#[test]
fn test_read_rtfd_includes_hidden_files() {
    let dir = tempdir().unwrap();
    let rtfd_dir = dir.path().join("test.rtfd");
    fs::create_dir(&rtfd_dir).unwrap();

    fs::write(rtfd_dir.join("TXT.rtf"), "test").unwrap();
    fs::write(rtfd_dir.join(".DS_Store"), b"mac metadata").unwrap();
    fs::write(rtfd_dir.join(".hidden"), b"hidden file").unwrap();

    let bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    // Current implementation includes hidden files as attachments
    assert_eq!(bundle.attachments.len(), 2);

    let filenames: Vec<String> = bundle
        .attachments
        .iter()
        .map(|a| a.filename.clone())
        .collect();
    assert!(filenames.contains(&".DS_Store".to_string()));
    assert!(filenames.contains(&".hidden".to_string()));
}
