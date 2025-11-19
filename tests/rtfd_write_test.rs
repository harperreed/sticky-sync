use sticky_situation::filesystem::rtfd::{RtfdBundle, Attachment};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_write_rtfd_bundle() {
    let dir = tempdir().unwrap();
    let rtfd_path = dir.path().join("test.rtfd");

    let bundle = RtfdBundle {
        rtf_data: b"{\\rtf1 Test}".to_vec(),
        attachments: vec![],
    };

    bundle.write(&rtfd_path).unwrap();

    assert!(rtfd_path.exists());
    assert!(rtfd_path.join("TXT.rtf").exists());

    let content = fs::read_to_string(rtfd_path.join("TXT.rtf")).unwrap();
    assert_eq!(content, "{\\rtf1 Test}");
}

#[test]
fn test_write_rtfd_with_attachments() {
    let dir = tempdir().unwrap();
    let rtfd_path = dir.path().join("test.rtfd");

    let bundle = RtfdBundle {
        rtf_data: b"{\\rtf1 Test}".to_vec(),
        attachments: vec![
            Attachment {
                filename: "IMG_001.tiff".to_string(),
                content: b"fake image".to_vec(),
            },
        ],
    };

    bundle.write(&rtfd_path).unwrap();

    assert!(rtfd_path.join("IMG_001.tiff").exists());
}
