// ABOUTME: Filesystem operations for reading/writing macOS Stickies data
// ABOUTME: Handles plist parsing and RTFD bundle I/O

pub mod plist;
pub mod rtfd;

pub use plist::StickyMetadata;
pub use rtfd::RtfdBundle;
