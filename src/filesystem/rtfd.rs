// ABOUTME: RTFD bundle reader/writer for macOS rich text format directories
// ABOUTME: Handles TXT.rtf files and embedded attachments like images

use crate::{Result, StickyError};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RtfdBundle {
    pub rtf_data: Vec<u8>,
    pub attachments: Vec<Attachment>,
}

impl RtfdBundle {
    pub fn read(rtfd_path: &Path) -> Result<Self> {
        if !rtfd_path.is_dir() {
            return Err(StickyError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "RTFD bundle not found",
            )));
        }

        let rtf_path = rtfd_path.join("TXT.rtf");
        let rtf_data = fs::read(&rtf_path)?;

        let mut attachments = Vec::new();

        for entry in fs::read_dir(rtfd_path)? {
            let entry = entry?;
            let filename = entry.file_name().to_string_lossy().to_string();

            // Skip TXT.rtf
            if filename == "TXT.rtf" {
                continue;
            }

            // Read any other files as attachments
            if entry.path().is_file() {
                let content = fs::read(entry.path())?;
                attachments.push(Attachment { filename, content });
            }
        }

        Ok(Self {
            rtf_data,
            attachments,
        })
    }

    pub fn modified_time(rtfd_path: &Path) -> Result<i64> {
        let rtf_path = rtfd_path.join("TXT.rtf");
        let metadata = fs::metadata(&rtf_path)?;
        let modified = metadata.modified()?;
        let timestamp = modified
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| StickyError::Io(std::io::Error::other(e)))?
            .as_secs() as i64;
        Ok(timestamp)
    }
}
