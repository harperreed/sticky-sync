// ABOUTME: Error types for sticky-situation using thiserror
// ABOUTME: Provides StickyError enum covering filesystem, database, and sync errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StickyError {
    #[error("Stickies directory not found: {0}")]
    StickiesNotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Plist error: {0}")]
    Plist(#[from] plist::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("RTF parsing error: {0}")]
    RtfParse(String),
}
