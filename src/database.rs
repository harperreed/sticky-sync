// ABOUTME: SQLite database operations for stickies storage and search
// ABOUTME: Handles schema creation, CRUD operations, and FTS5 full-text search

use crate::Result;
use rusqlite::Connection;
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
