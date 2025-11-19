// ABOUTME: SQLite database operations for stickies storage and search
// ABOUTME: Handles schema creation, CRUD operations, and FTS5 full-text search

use crate::Result;
use rusqlite::{Connection, params, OptionalExtension};
use std::path::Path;

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

        // Create FTS5 virtual table with its own content
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS stickies_fts USING fts5(
                uuid UNINDEXED,
                content_text
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn insert_sticky(&self, sticky: &Sticky) -> Result<()> {
        // Delete from FTS index first to avoid corruption
        self.conn.execute(
            "DELETE FROM stickies_fts WHERE uuid = ?1",
            params![&sticky.uuid],
        )?;

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

        // Insert into FTS index
        self.conn.execute(
            "INSERT INTO stickies_fts (uuid, content_text) VALUES (?1, ?2)",
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
