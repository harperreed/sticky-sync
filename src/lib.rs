// ABOUTME: Library root for sticky-situation - macOS Stickies sync tool
// ABOUTME: Exports public modules for configuration, database, filesystem, and sync

pub mod config;
pub mod database;
pub mod filesystem;
pub mod sync;
pub mod rtf;
pub mod error;

pub use error::StickyError;
pub type Result<T> = std::result::Result<T, StickyError>;
