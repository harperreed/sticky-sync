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

    pub fn write(&self, rtfd_path: &Path) -> Result<()> {
        fs::create_dir_all(rtfd_path)?;

        let rtf_path = rtfd_path.join("TXT.rtf");
        fs::write(&rtf_path, &self.rtf_data)?;

        for attachment in &self.attachments {
            let attachment_path = rtfd_path.join(&attachment.filename);
            fs::write(&attachment_path, &attachment.content)?;
        }

        Ok(())
    }

    pub fn create_minimal(text: &str) -> Self {
        let rtf_data = format!(
            "{{\\rtf1\\ansi\\ansicpg1252\\cocoartf2820\n\
             {{\\fonttbl\\f0\\fswiss\\fcharset0 Helvetica;}}\n\
             {{\\colortbl;\\red255\\green255\\blue255;}}\n\
             \\pard\\tx560\\tx1120\\tx1680\\tx2240\\tx2800\\tx3360\\tx3920\\tx4480\\tx5040\\tx5600\\tx6160\\tx6720\\pardirnatural\\partightenfactor0\n\
             \\f0\\fs24 \\cf0 {}}}",
            text
        );

        Self {
            rtf_data: rtf_data.into_bytes(),
            attachments: vec![],
        }
    }
}
