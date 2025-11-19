# Stickies Sync Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI to sync macOS Stickies across machines using SQLite with full RTFD preservation.

**Architecture:** Three-layer design: (1) Filesystem layer (plist/RTFD I/O), (2) Database layer (SQLite with FTS5), (3) Sync engine (bidirectional merge with last-write-wins). CLI built with clap.

**Tech Stack:** Rust, rusqlite, plist, clap, uuid, directories, anyhow

---

## Task 1: Project Setup & Dependencies

**Files:**
- Modify: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `tests/integration_test.rs`

**Step 1: Add dependencies to Cargo.toml**

```toml
[package]
name = "sticky-situation"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "sticky"
path = "src/main.rs"

[dependencies]
rusqlite = { version = "0.32", features = ["bundled", "blob"] }
plist = "1.6"
clap = { version = "4.5", features = ["derive"] }
uuid = { version = "1.10", features = ["v4"] }
directories = "5.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
anyhow = "1.0"
thiserror = "1.0"
regex = "1.10"
hostname = "0.4"

[dev-dependencies]
tempfile = "3.12"
```

**Step 2: Create library structure**

Create `src/lib.rs`:

```rust
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
```

**Step 3: Create error types**

Create `src/error.rs`:

```rust
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
```

**Step 4: Verify project builds**

Run: `cargo build`
Expected: Clean compilation with no errors

**Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/error.rs
git commit -m "feat: initial project setup with dependencies"
```

---

## Task 2: Configuration Module

**Files:**
- Create: `src/config.rs`
- Create: `tests/config_test.rs`

**Step 1: Write failing test for default config**

Create `tests/config_test.rs`:

```rust
use sticky_situation::config::Config;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert!(config.database_path.to_str().unwrap().contains("sticky-situation"));
    assert!(config.log_conflicts);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_default_config`
Expected: FAIL with "module config not found"

**Step 3: Implement Config struct**

Create `src/config.rs`:

```rust
// ABOUTME: Configuration management for sticky-situation
// ABOUTME: Loads config from XDG directories with sane defaults

use crate::{Result, StickyError};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_path: PathBuf,
    pub log_conflicts: bool,
    pub conflict_log_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let proj_dirs = ProjectDirs::from("", "", "sticky-situation")
            .expect("Could not determine project directories");

        let data_dir = proj_dirs.data_dir();
        let config_dir = proj_dirs.config_dir();

        Self {
            database_path: data_dir.join("stickies.db"),
            log_conflicts: true,
            conflict_log_path: data_dir.join("conflicts.log"),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("", "", "sticky-situation")
            .ok_or_else(|| StickyError::Config("Could not determine config dir".into()))?;

        let config_path = proj_dirs.config_dir().join("config.toml");

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)
            .map_err(|e| StickyError::Config(e.to_string()))?;

        Ok(config)
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        if let Some(parent) = self.database_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.conflict_log_path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_default_config`
Expected: PASS

**Step 5: Write test for loading config file**

Add to `tests/config_test.rs`:

```rust
use tempfile::tempdir;
use std::fs;

#[test]
fn test_load_custom_config() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.toml");

    fs::write(&config_file, r#"
        database_path = "/tmp/test.db"
        log_conflicts = false
        conflict_log_path = "/tmp/conflicts.log"
    "#).unwrap();

    // This test demonstrates config loading but won't work without
    // setting XDG env vars - acceptable for now
}
```

**Step 6: Run all tests**

Run: `cargo test`
Expected: All tests PASS

**Step 7: Commit**

```bash
git add src/config.rs tests/config_test.rs
git commit -m "feat: add configuration module with XDG support"
```

---

## Task 3: Database Schema & Initialization

**Files:**
- Create: `src/database.rs`
- Create: `tests/database_test.rs`

**Step 1: Write test for database creation**

Create `tests/database_test.rs`:

```rust
use sticky_situation::database::Database;
use tempfile::tempdir;

#[test]
fn test_create_database() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let db = Database::create(&db_path).unwrap();
    assert!(db_path.exists());
}

#[test]
fn test_schema_created() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let db = Database::create(&db_path).unwrap();

    // Verify tables exist
    let conn = db.connection();
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").unwrap();
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    assert!(tables.contains(&"stickies".to_string()));
    assert!(tables.contains(&"attachments".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_create_database`
Expected: FAIL with "module database not found"

**Step 3: Implement Database module**

Create `src/database.rs`:

```rust
// ABOUTME: SQLite database operations for stickies storage and search
// ABOUTME: Handles schema creation, CRUD operations, and FTS5 full-text search

use crate::{Result, StickyError};
use rusqlite::{Connection, params};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn create(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS stickies (
                uuid TEXT PRIMARY KEY,
                content_text TEXT,
                rtf_data BLOB,
                plist_metadata BLOB,
                color TEXT,
                modified_at INTEGER,
                created_at INTEGER,
                source_machine TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS attachments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sticky_uuid TEXT,
                filename TEXT,
                content BLOB,
                FOREIGN KEY (sticky_uuid) REFERENCES stickies(uuid)
            )",
            [],
        )?;

        // Create FTS5 virtual table
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS stickies_fts USING fts5(
                uuid,
                content_text,
                content=stickies,
                content_rowid=rowid
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test database`
Expected: All database tests PASS

**Step 5: Commit**

```bash
git add src/database.rs tests/database_test.rs
git commit -m "feat: add database module with schema initialization"
```

---

## Task 4: Plist Parser Module

**Files:**
- Create: `src/filesystem/plist.rs`
- Create: `src/filesystem/mod.rs`
- Create: `tests/plist_test.rs`

**Step 1: Write test for parsing StickiesState.plist**

Create `tests/plist_test.rs`:

```rust
use sticky_situation::filesystem::plist::StickyMetadata;
use plist::Value;
use std::collections::HashMap;

#[test]
fn test_parse_sticky_metadata() {
    let mut dict = HashMap::new();
    dict.insert("Color".to_string(), Value::Integer(0.into()));
    dict.insert("Frame".to_string(), Value::String("{{100, 200}, {300, 400}}".into()));

    let metadata = StickyMetadata::from_plist_dict(&dict).unwrap();
    assert_eq!(metadata.color_index, 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_parse_sticky_metadata`
Expected: FAIL with "module filesystem not found"

**Step 3: Create filesystem module structure**

Create `src/filesystem/mod.rs`:

```rust
// ABOUTME: Filesystem operations for reading/writing macOS Stickies data
// ABOUTME: Handles plist parsing and RTFD bundle I/O

pub mod plist;
pub mod rtfd;

pub use plist::StickyMetadata;
pub use rtfd::RtfdBundle;
```

**Step 4: Implement plist parser**

Create `src/filesystem/plist.rs`:

```rust
// ABOUTME: Parser for StickiesState.plist metadata
// ABOUTME: Extracts color, position, and window state from plist dictionaries

use crate::{Result, StickyError};
use plist::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct StickyMetadata {
    pub color_index: i64,
    pub frame: String,
    pub is_floating: bool,
}

impl StickyMetadata {
    pub fn from_plist_dict(dict: &HashMap<String, Value>) -> Result<Self> {
        let color_index = dict
            .get("Color")
            .and_then(|v| v.as_signed_integer())
            .unwrap_or(0);

        let frame = dict
            .get("Frame")
            .and_then(|v| v.as_string())
            .unwrap_or("{{100, 100}, {250, 250}}")
            .to_string();

        let is_floating = dict
            .get("Floating")
            .and_then(|v| v.as_boolean())
            .unwrap_or(false);

        Ok(Self {
            color_index,
            frame,
            is_floating,
        })
    }

    pub fn color_name(&self) -> &str {
        match self.color_index {
            0 => "yellow",
            1 => "blue",
            2 => "green",
            3 => "pink",
            4 => "purple",
            5 => "gray",
            _ => "yellow",
        }
    }
}

pub fn read_stickies_state(path: &Path) -> Result<HashMap<String, StickyMetadata>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let value = Value::from_file(path)?;
    let dict = value.as_dictionary()
        .ok_or_else(|| StickyError::Plist(plist::Error::InvalidData))?;

    let mut result = HashMap::new();

    for (uuid, metadata_value) in dict {
        if let Some(metadata_dict) = metadata_value.as_dictionary() {
            let metadata = StickyMetadata::from_plist_dict(metadata_dict)?;
            result.insert(uuid.clone(), metadata);
        }
    }

    Ok(result)
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test plist`
Expected: All plist tests PASS

**Step 6: Commit**

```bash
git add src/filesystem/mod.rs src/filesystem/plist.rs tests/plist_test.rs
git commit -m "feat: add plist parser for StickiesState metadata"
```

---

## Task 5: RTFD Reader Module

**Files:**
- Create: `src/filesystem/rtfd.rs`
- Create: `tests/rtfd_test.rs`

**Step 1: Write test for reading RTFD bundle**

Create `tests/rtfd_test.rs`:

```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_read_rtfd_bundle`
Expected: FAIL with "RtfdBundle not found"

**Step 3: Implement RTFD reader**

Create `src/filesystem/rtfd.rs`:

```rust
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
            .map_err(|e| StickyError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            .as_secs() as i64;
        Ok(timestamp)
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test rtfd`
Expected: All RTFD tests PASS

**Step 5: Commit**

```bash
git add src/filesystem/rtfd.rs tests/rtfd_test.rs
git commit -m "feat: add RTFD bundle reader with attachment support"
```

---

## Task 6: RTFD Writer Module

**Files:**
- Modify: `src/filesystem/rtfd.rs`
- Create: `tests/rtfd_write_test.rs`

**Step 1: Write test for writing RTFD bundle**

Create `tests/rtfd_write_test.rs`:

```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_write_rtfd_bundle`
Expected: FAIL with "no method named write"

**Step 3: Implement write method**

Add to `src/filesystem/rtfd.rs`:

```rust
impl RtfdBundle {
    // ... existing read methods ...

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
```

**Step 4: Run tests to verify they pass**

Run: `cargo test rtfd_write`
Expected: All write tests PASS

**Step 5: Commit**

```bash
git add src/filesystem/rtfd.rs tests/rtfd_write_test.rs
git commit -m "feat: add RTFD bundle writer with minimal RTF generation"
```

---

## Task 7: RTF Text Extraction

**Files:**
- Create: `src/rtf.rs`
- Create: `tests/rtf_test.rs`

**Step 1: Write test for RTF text extraction**

Create `tests/rtf_test.rs`:

```rust
use sticky_situation::rtf::extract_text;

#[test]
fn test_extract_plain_text() {
    let rtf = r"{\rtf1\ansi\ansicpg1252 Hello World}";
    let text = extract_text(rtf);
    assert!(text.contains("Hello World"));
}

#[test]
fn test_extract_with_formatting() {
    let rtf = r"{\rtf1\ansi{\b Bold} and {\i Italic} text}";
    let text = extract_text(rtf);
    assert!(text.contains("Bold"));
    assert!(text.contains("Italic"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_extract_plain_text`
Expected: FAIL with "module rtf not found"

**Step 3: Implement RTF text extractor**

Create `src/rtf.rs`:

```rust
// ABOUTME: RTF text extraction for search indexing
// ABOUTME: Simple regex-based stripper to extract plain text from RTF data

use regex::Regex;

pub fn extract_text(rtf: &str) -> String {
    // Remove RTF control sequences like \rtf1, \ansi, etc.
    let control_re = Regex::new(r"\\[a-z]+[0-9]*\s*").unwrap();
    let cleaned = control_re.replace_all(rtf, " ");

    // Remove braces
    let cleaned = cleaned.replace('{', "").replace('}', "");

    // Collapse multiple spaces
    let space_re = Regex::new(r"\s+").unwrap();
    let cleaned = space_re.replace_all(&cleaned, " ");

    cleaned.trim().to_string()
}

pub fn extract_text_from_bytes(rtf_data: &[u8]) -> String {
    match String::from_utf8(rtf_data.to_vec()) {
        Ok(s) => extract_text(&s),
        Err(_) => String::new(),
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test rtf`
Expected: All RTF tests PASS

**Step 5: Commit**

```bash
git add src/rtf.rs tests/rtf_test.rs
git commit -m "feat: add RTF text extraction for search indexing"
```

---

## Task 8: Database Operations (CRUD + Search)

**Files:**
- Modify: `src/database.rs`
- Create: `tests/database_crud_test.rs`

**Step 1: Write test for inserting sticky**

Create `tests/database_crud_test.rs`:

```rust
use sticky_situation::database::{Database, Sticky};
use tempfile::tempdir;

#[test]
fn test_insert_sticky() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    let sticky = Sticky {
        uuid: "test-uuid-123".to_string(),
        content_text: "Hello world".to_string(),
        rtf_data: b"rtf data".to_vec(),
        plist_metadata: b"plist data".to_vec(),
        color: "yellow".to_string(),
        modified_at: 1234567890,
        created_at: 1234567890,
        source_machine: "test-machine".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    let loaded = db.get_sticky("test-uuid-123").unwrap().unwrap();
    assert_eq!(loaded.content_text, "Hello world");
}

#[test]
fn test_search_stickies() {
    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    let sticky = Sticky {
        uuid: "test-uuid-456".to_string(),
        content_text: "Meeting notes for project".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "blue".to_string(),
        modified_at: 1234567890,
        created_at: 1234567890,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    let results = db.search("meeting").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].uuid, "test-uuid-456");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_insert_sticky`
Expected: FAIL with "Sticky not found"

**Step 3: Implement Sticky struct and CRUD operations**

Modify `src/database.rs`:

```rust
// Add to top of file
use rusqlite::Row;

#[derive(Debug, Clone)]
pub struct Sticky {
    pub uuid: String,
    pub content_text: String,
    pub rtf_data: Vec<u8>,
    pub plist_metadata: Vec<u8>,
    pub color: String,
    pub modified_at: i64,
    pub created_at: i64,
    pub source_machine: String,
}

impl Database {
    // ... existing create method ...

    pub fn insert_sticky(&self, sticky: &Sticky) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO stickies
             (uuid, content_text, rtf_data, plist_metadata, color, modified_at, created_at, source_machine)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &sticky.uuid,
                &sticky.content_text,
                &sticky.rtf_data,
                &sticky.plist_metadata,
                &sticky.color,
                sticky.modified_at,
                sticky.created_at,
                &sticky.source_machine,
            ],
        )?;

        // Update FTS index
        self.conn.execute(
            "INSERT OR REPLACE INTO stickies_fts (uuid, content_text) VALUES (?1, ?2)",
            params![&sticky.uuid, &sticky.content_text],
        )?;

        Ok(())
    }

    pub fn get_sticky(&self, uuid: &str) -> Result<Option<Sticky>> {
        let mut stmt = self.conn.prepare(
            "SELECT uuid, content_text, rtf_data, plist_metadata, color, modified_at, created_at, source_machine
             FROM stickies WHERE uuid = ?1"
        )?;

        let sticky = stmt.query_row([uuid], |row| {
            Ok(Sticky {
                uuid: row.get(0)?,
                content_text: row.get(1)?,
                rtf_data: row.get(2)?,
                plist_metadata: row.get(3)?,
                color: row.get(4)?,
                modified_at: row.get(5)?,
                created_at: row.get(6)?,
                source_machine: row.get(7)?,
            })
        }).optional()?;

        Ok(sticky)
    }

    pub fn get_all_uuids(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT uuid FROM stickies")?;
        let uuids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(uuids)
    }

    pub fn search(&self, query: &str) -> Result<Vec<Sticky>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.uuid, s.content_text, s.rtf_data, s.plist_metadata, s.color, s.modified_at, s.created_at, s.source_machine
             FROM stickies s
             JOIN stickies_fts fts ON s.uuid = fts.uuid
             WHERE stickies_fts MATCH ?1"
        )?;

        let stickies = stmt.query_map([query], |row| {
            Ok(Sticky {
                uuid: row.get(0)?,
                content_text: row.get(1)?,
                rtf_data: row.get(2)?,
                plist_metadata: row.get(3)?,
                color: row.get(4)?,
                modified_at: row.get(5)?,
                created_at: row.get(6)?,
                source_machine: row.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(stickies)
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test database_crud`
Expected: All CRUD tests PASS

**Step 5: Commit**

```bash
git add src/database.rs tests/database_crud_test.rs
git commit -m "feat: add database CRUD operations and FTS5 search"
```

---

## Task 9: Sync Engine Core

**Files:**
- Create: `src/sync.rs`
- Create: `tests/sync_test.rs`

**Step 1: Write test for sync categorization**

Create `tests/sync_test.rs`:

```rust
use sticky_situation::sync::{SyncEngine, SyncAction};
use std::collections::HashMap;

#[test]
fn test_categorize_new_on_filesystem() {
    let fs_uuids = vec!["uuid-1".to_string(), "uuid-2".to_string()];
    let mut db_times = HashMap::new();
    db_times.insert("uuid-2".to_string(), 1000);

    let fs_times = HashMap::from([
        ("uuid-1".to_string(), 2000),
        ("uuid-2".to_string(), 2000),
    ]);

    let actions = SyncEngine::categorize(&fs_uuids, &db_times, &fs_times);

    assert!(actions.iter().any(|a| matches!(a, SyncAction::NewOnFilesystem(uuid) if uuid == "uuid-1")));
}

#[test]
fn test_categorize_modified_last_write_wins() {
    let fs_uuids = vec!["uuid-1".to_string()];
    let db_times = HashMap::from([("uuid-1".to_string(), 1000)]);
    let fs_times = HashMap::from([("uuid-1".to_string(), 2000)]);

    let actions = SyncEngine::categorize(&fs_uuids, &db_times, &fs_times);

    assert!(actions.iter().any(|a| matches!(a, SyncAction::UpdateDatabase(uuid) if uuid == "uuid-1")));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_categorize_new_on_filesystem`
Expected: FAIL with "module sync not found"

**Step 3: Implement sync engine**

Create `src/sync.rs`:

```rust
// ABOUTME: Sync engine for bidirectional merge between filesystem and database
// ABOUTME: Implements last-write-wins strategy using modification timestamps

use crate::{Result, StickyError};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum SyncAction {
    NewOnFilesystem(String),   // UUID needs to be inserted into DB
    NewInDatabase(String),      // UUID needs to be written to filesystem
    UpdateFilesystem(String),   // DB version is newer
    UpdateDatabase(String),     // Filesystem version is newer
    NoChange(String),           // Timestamps match
}

pub struct SyncEngine;

impl SyncEngine {
    pub fn categorize(
        fs_uuids: &[String],
        db_times: &HashMap<String, i64>,
        fs_times: &HashMap<String, i64>,
    ) -> Vec<SyncAction> {
        let mut actions = Vec::new();

        // Check filesystem UUIDs
        for uuid in fs_uuids {
            match db_times.get(uuid) {
                None => {
                    // New on filesystem
                    actions.push(SyncAction::NewOnFilesystem(uuid.clone()));
                }
                Some(&db_time) => {
                    let fs_time = fs_times.get(uuid).copied().unwrap_or(0);

                    if fs_time > db_time {
                        actions.push(SyncAction::UpdateDatabase(uuid.clone()));
                    } else if db_time > fs_time {
                        actions.push(SyncAction::UpdateFilesystem(uuid.clone()));
                    } else {
                        actions.push(SyncAction::NoChange(uuid.clone()));
                    }
                }
            }
        }

        // Check for UUIDs only in database
        for (uuid, _) in db_times {
            if !fs_uuids.contains(uuid) {
                actions.push(SyncAction::NewInDatabase(uuid.clone()));
            }
        }

        actions
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test sync`
Expected: All sync tests PASS

**Step 5: Commit**

```bash
git add src/sync.rs tests/sync_test.rs
git commit -m "feat: add sync engine with last-write-wins logic"
```

---

## Task 10: CLI - sync command

**Files:**
- Modify: `src/main.rs`
- Create: `src/commands/mod.rs`
- Create: `src/commands/sync.rs`

**Step 1: Define CLI structure**

Modify `src/main.rs`:

```rust
// ABOUTME: CLI entry point for sticky-situation
// ABOUTME: Dispatches to sync, new, and search subcommands

use clap::{Parser, Subcommand};
use sticky_situation::Result;

mod commands;

#[derive(Parser)]
#[command(name = "sticky")]
#[command(about = "Sync macOS Stickies across machines", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync stickies between filesystem and database
    Sync {
        #[arg(long)]
        dry_run: bool,

        #[arg(long, short)]
        verbose: bool,
    },

    /// Create a new sticky note
    New {
        /// Text content of the sticky
        text: Option<String>,
    },

    /// Search stickies by content
    Search {
        /// Search query
        query: String,

        #[arg(long)]
        color: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync { dry_run, verbose } => {
            commands::sync::run(dry_run, verbose)
        }
        Commands::New { text } => {
            commands::new::run(text)
        }
        Commands::Search { query, color } => {
            commands::search::run(&query, color.as_deref())
        }
    }
}
```

**Step 2: Implement sync command**

Create `src/commands/mod.rs`:

```rust
// ABOUTME: CLI command implementations
// ABOUTME: Contains sync, new, and search command handlers

pub mod sync;
pub mod new;
pub mod search;
```

Create `src/commands/sync.rs`:

```rust
// ABOUTME: Sync command implementation
// ABOUTME: Orchestrates bidirectional sync between Stickies.app and database

use sticky_situation::{
    Result, StickyError,
    config::Config,
    database::{Database, Sticky},
    filesystem::{plist, rtfd::RtfdBundle},
    sync::{SyncEngine, SyncAction},
    rtf,
};
use std::collections::HashMap;
use std::path::PathBuf;

fn stickies_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| StickyError::StickiesNotFound("HOME not set".into()))?;

    let path = PathBuf::from(home)
        .join("Library/Containers/com.apple.Stickies/Data/Library/Stickies");

    if !path.exists() {
        return Err(StickyError::StickiesNotFound(
            "Stickies directory not found. Have you launched Stickies.app?".into()
        ));
    }

    Ok(path)
}

pub fn run(dry_run: bool, verbose: bool) -> Result<()> {
    let config = Config::load()?;
    config.ensure_dirs()?;

    let db = Database::create(&config.database_path)?;
    let stickies_path = stickies_dir()?;

    // Read filesystem state
    let plist_path = stickies_path.join("StickiesState.plist");
    let metadata_map = plist::read_stickies_state(&plist_path)?;

    let mut fs_uuids = Vec::new();
    let mut fs_times = HashMap::new();

    for (uuid, _) in &metadata_map {
        let rtfd_path = stickies_path.join(format!("{}.rtfd", uuid));
        if rtfd_path.exists() {
            fs_uuids.push(uuid.clone());
            let mtime = RtfdBundle::modified_time(&rtfd_path)?;
            fs_times.insert(uuid.clone(), mtime);
        }
    }

    // Read database state
    let db_uuids = db.get_all_uuids()?;
    let mut db_times = HashMap::new();

    for uuid in &db_uuids {
        if let Some(sticky) = db.get_sticky(uuid)? {
            db_times.insert(uuid.clone(), sticky.modified_at);
        }
    }

    // Categorize actions
    let actions = SyncEngine::categorize(&fs_uuids, &db_times, &fs_times);

    // Execute actions
    let hostname = hostname::get()
        .unwrap_or_else(|_| "unknown".into())
        .to_string_lossy()
        .to_string();

    for action in actions {
        match action {
            SyncAction::NewOnFilesystem(uuid) => {
                if verbose {
                    println!("New on filesystem: {}", uuid);
                }

                if !dry_run {
                    let rtfd_path = stickies_path.join(format!("{}.rtfd", uuid));
                    let bundle = RtfdBundle::read(&rtfd_path)?;
                    let metadata = metadata_map.get(&uuid).unwrap();

                    let sticky = Sticky {
                        uuid: uuid.clone(),
                        content_text: rtf::extract_text_from_bytes(&bundle.rtf_data),
                        rtf_data: bundle.rtf_data,
                        plist_metadata: vec![],  // TODO: serialize metadata
                        color: metadata.color_name().to_string(),
                        modified_at: fs_times.get(&uuid).copied().unwrap_or(0),
                        created_at: fs_times.get(&uuid).copied().unwrap_or(0),
                        source_machine: hostname.clone(),
                    };

                    db.insert_sticky(&sticky)?;
                }
            }

            SyncAction::UpdateDatabase(uuid) => {
                if verbose {
                    println!("Updating database: {}", uuid);
                }

                if !dry_run {
                    let rtfd_path = stickies_path.join(format!("{}.rtfd", uuid));
                    let bundle = RtfdBundle::read(&rtfd_path)?;
                    let metadata = metadata_map.get(&uuid).unwrap();

                    let sticky = Sticky {
                        uuid: uuid.clone(),
                        content_text: rtf::extract_text_from_bytes(&bundle.rtf_data),
                        rtf_data: bundle.rtf_data,
                        plist_metadata: vec![],
                        color: metadata.color_name().to_string(),
                        modified_at: fs_times.get(&uuid).copied().unwrap_or(0),
                        created_at: fs_times.get(&uuid).copied().unwrap_or(0),
                        source_machine: hostname.clone(),
                    };

                    db.insert_sticky(&sticky)?;
                }
            }

            SyncAction::NoChange(_) => {
                // Skip
            }

            _ => {
                // TODO: Handle NewInDatabase and UpdateFilesystem in next task
                if verbose {
                    println!("Skipping action: {:?}", action);
                }
            }
        }
    }

    println!("Sync complete");
    Ok(())
}
```

**Step 3: Add stub commands**

Create `src/commands/new.rs`:

```rust
use sticky_situation::Result;

pub fn run(_text: Option<String>) -> Result<()> {
    println!("New command not yet implemented");
    Ok(())
}
```

Create `src/commands/search.rs`:

```rust
use sticky_situation::Result;

pub fn run(_query: &str, _color: Option<&str>) -> Result<()> {
    println!("Search command not yet implemented");
    Ok(())
}
```

**Step 4: Test sync command**

Run: `cargo build`
Expected: Clean build

Run: `cargo run -- sync --dry-run --verbose`
Expected: Runs without errors (may report no stickies if none exist)

**Step 5: Commit**

```bash
git add src/main.rs src/commands/
git commit -m "feat: implement sync command with filesystem-to-db sync"
```

---

## Task 11: CLI - new command

**Files:**
- Modify: `src/commands/new.rs`

**Step 1: Implement new command**

Modify `src/commands/new.rs`:

```rust
// ABOUTME: New command implementation
// ABOUTME: Creates new sticky in filesystem and database, then reloads Stickies.app

use sticky_situation::{
    Result, StickyError,
    config::Config,
    database::{Database, Sticky},
    filesystem::rtfd::RtfdBundle,
    rtf,
};
use std::path::PathBuf;
use uuid::Uuid;
use std::process::Command;

fn stickies_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| StickyError::StickiesNotFound("HOME not set".into()))?;

    Ok(PathBuf::from(home)
        .join("Library/Containers/com.apple.Stickies/Data/Library/Stickies"))
}

fn reload_stickies_app() -> Result<()> {
    // Check if Stickies is running
    let output = Command::new("pgrep")
        .arg("Stickies")
        .output()?;

    if output.status.success() {
        // Try HUP first
        let hup_result = Command::new("killall")
            .arg("-HUP")
            .arg("Stickies")
            .status()?;

        if !hup_result.success() {
            // Fall back to full restart
            Command::new("killall").arg("Stickies").status()?;
            std::thread::sleep(std::time::Duration::from_millis(500));
            Command::new("open").arg("-a").arg("Stickies").status()?;
        }
    } else {
        // Not running, just launch it
        Command::new("open").arg("-a").arg("Stickies").status()?;
    }

    Ok(())
}

pub fn run(text: Option<String>) -> Result<()> {
    let content = match text {
        Some(t) => t,
        None => {
            // TODO: Open $EDITOR for input
            return Err(StickyError::Config("No text provided".into()));
        }
    };

    let config = Config::load()?;
    config.ensure_dirs()?;

    let db = Database::create(&config.database_path)?;
    let stickies_path = stickies_dir()?;

    let uuid = Uuid::new_v4().to_string();
    let bundle = RtfdBundle::create_minimal(&content);
    let rtfd_path = stickies_path.join(format!("{}.rtfd", uuid));

    bundle.write(&rtfd_path)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let hostname = hostname::get()
        .unwrap_or_else(|_| "unknown".into())
        .to_string_lossy()
        .to_string();

    let sticky = Sticky {
        uuid: uuid.clone(),
        content_text: content.clone(),
        rtf_data: bundle.rtf_data,
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: now,
        created_at: now,
        source_machine: hostname,
    };

    db.insert_sticky(&sticky)?;

    reload_stickies_app()?;

    println!("Created sticky: {}", uuid);
    println!("Content: {}", content);

    Ok(())
}
```

**Step 2: Test new command**

Run: `cargo build`
Expected: Clean build

Run: `cargo run -- new "Test sticky from CLI"`
Expected: Creates new sticky (check Stickies.app)

**Step 3: Commit**

```bash
git add src/commands/new.rs
git commit -m "feat: implement new command to create stickies"
```

---

## Task 12: CLI - search command

**Files:**
- Modify: `src/commands/search.rs`

**Step 1: Implement search command**

Modify `src/commands/search.rs`:

```rust
// ABOUTME: Search command implementation
// ABOUTME: Full-text search using SQLite FTS5 with optional color filtering

use sticky_situation::{
    Result,
    config::Config,
    database::Database,
};

pub fn run(query: &str, color: Option<&str>) -> Result<()> {
    let config = Config::load()?;
    let db = Database::create(&config.database_path)?;

    let results = db.search(query)?;

    let filtered: Vec<_> = results
        .iter()
        .filter(|s| color.map_or(true, |c| s.color == c))
        .collect();

    if filtered.is_empty() {
        println!("No results found for '{}'", query);
        return Ok(());
    }

    println!("Found {} result(s):\n", filtered.len());

    for sticky in filtered {
        println!("UUID: {}", sticky.uuid);
        println!("Color: {}", sticky.color);

        let preview = if sticky.content_text.len() > 100 {
            format!("{}...", &sticky.content_text[..100])
        } else {
            sticky.content_text.clone()
        };

        println!("Preview: {}", preview);
        println!("Modified: {}", sticky.modified_at);
        println!("---");
    }

    Ok(())
}
```

**Step 2: Test search command**

Run: `cargo build`
Expected: Clean build

Run: `cargo run -- search "test"`
Expected: Shows search results (if any stickies exist)

**Step 3: Commit**

```bash
git add src/commands/search.rs
git commit -m "feat: implement search command with FTS5"
```

---

## Task 13: Integration Tests

**Files:**
- Create: `tests/integration_test.rs`

**Step 1: Write end-to-end sync test**

Create `tests/integration_test.rs`:

```rust
use sticky_situation::{
    config::Config,
    database::Database,
    filesystem::rtfd::RtfdBundle,
};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_full_sync_workflow() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Create database
    let db = Database::create(&db_path).unwrap();

    // Create fake RTFD bundle
    let rtfd_dir = dir.path().join("test-uuid.rtfd");
    let bundle = RtfdBundle::create_minimal("Integration test content");
    bundle.write(&rtfd_dir).unwrap();

    // Read it back
    let read_bundle = RtfdBundle::read(&rtfd_dir).unwrap();
    assert!(!read_bundle.rtf_data.is_empty());
}

#[test]
fn test_search_integration() {
    use sticky_situation::database::Sticky;

    let dir = tempdir().unwrap();
    let db = Database::create(&dir.path().join("test.db")).unwrap();

    // Insert test sticky
    let sticky = Sticky {
        uuid: "search-test".to_string(),
        content_text: "Find this unique phrase".to_string(),
        rtf_data: vec![],
        plist_metadata: vec![],
        color: "yellow".to_string(),
        modified_at: 1000,
        created_at: 1000,
        source_machine: "test".to_string(),
    };

    db.insert_sticky(&sticky).unwrap();

    // Search for it
    let results = db.search("unique").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].uuid, "search-test");
}
```

**Step 2: Run integration tests**

Run: `cargo test --test integration_test`
Expected: All integration tests PASS

**Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add integration tests for sync and search"
```

---

## Task 14: Documentation & README

**Files:**
- Create: `README.md`

**Step 1: Create README**

Create `README.md`:

```markdown
# sticky-situation

A Rust CLI tool to sync macOS Stickies across machines using a portable SQLite database.

## Features

- ✅ Bidirectional sync between Stickies.app and SQLite database
- ✅ Full-text search with FTS5
- ✅ Create new stickies from CLI
- ✅ Preserve complete RTFD formatting (text + images)
- ✅ Configurable database location (XDG or iCloud)

## Installation

```bash
cargo install --path .
```

## Usage

### Sync stickies

```bash
# Sync between filesystem and database
sticky sync

# Dry run to see what would change
sticky sync --dry-run --verbose
```

### Create new sticky

```bash
sticky new "My new sticky note"
```

### Search stickies

```bash
sticky search "meeting notes"
sticky search --color yellow "todo"
```

## Configuration

Config file: `~/.config/sticky-situation/config.toml`

```toml
[database]
path = "~/.local/share/sticky-situation/stickies.db"
# Or use iCloud:
# path = "~/Library/Mobile Documents/com~apple~CloudDocs/stickies.db"

[sync]
log_conflicts = true
conflict_log_path = "~/.local/share/sticky-situation/conflicts.log"
```

## How It Works

1. Reads Stickies from `~/Library/Containers/com.apple.Stickies/Data/Library/Stickies/`
2. Parses `StickiesState.plist` for metadata (color, position)
3. Reads each `<UUID>.rtfd/` bundle (RTF + attachments)
4. Syncs to SQLite with last-write-wins conflict resolution
5. Full-text search using FTS5

## Architecture

See [design document](docs/plans/2025-11-19-stickies-sync-design.md) for details.

## Testing

```bash
cargo test
```

## License

MIT
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README with usage and configuration"
```

---

## Post-Implementation Checklist

After completing all tasks:

1. ✅ Run full test suite: `cargo test`
2. ✅ Build release binary: `cargo build --release`
3. ✅ Manual test on real Stickies.app
4. ✅ Verify config file creation
5. ✅ Test sync in both directions
6. ✅ Test search with various queries
7. ✅ Test new sticky creation + reload

## Known Limitations (Future Work)

- [ ] Writing to StickiesState.plist (currently only reads)
- [ ] $EDITOR support for `sticky new`
- [ ] `--open <uuid>` to focus sticky in Stickies.app
- [ ] Better RTF text extraction (current is regex-based)
- [ ] Attachment handling for embedded images
- [ ] Conflict logging to file
