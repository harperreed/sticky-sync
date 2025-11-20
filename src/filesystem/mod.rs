// ABOUTME: Filesystem operations for reading/writing macOS Stickies data
// ABOUTME: Handles plist parsing and RTFD bundle I/O

use crate::{Result, StickyError};
use std::path::PathBuf;

pub mod plist;
pub mod rtfd;

pub use plist::StickyMetadata;
pub use rtfd::RtfdBundle;

/// Get the path to the Stickies directory, validating it exists
pub fn stickies_dir() -> Result<PathBuf> {
    let home =
        std::env::var("HOME").map_err(|_| StickyError::StickiesNotFound("HOME not set".into()))?;

    let path =
        PathBuf::from(home).join("Library/Containers/com.apple.Stickies/Data/Library/Stickies");

    if !path.exists() {
        return Err(StickyError::StickiesNotFound(
            "Stickies directory not found. Have you launched Stickies.app?".into(),
        ));
    }

    Ok(path)
}
