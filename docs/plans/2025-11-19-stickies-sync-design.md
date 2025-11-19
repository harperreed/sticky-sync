# Stickies Sync CLI Design

**Date:** 2025-11-19
**Status:** Approved

## Overview

A Rust CLI tool to sync macOS Stickies across machines using a portable SQLite database. Enables robust backup, search, and cross-machine synchronization of sticky notes.

## Goals

1. Sync stickies bidirectionally between filesystem and database
2. Support database location in XDG directories or iCloud Drive
3. Create new stickies from CLI
4. Full-text search across all stickies
5. Preserve complete RTFD formatting for cross-machine restoration

## High-Level Architecture

### Core Components

1. **Stickies Parser**
   - Reads from `~/Library/Containers/com.apple.Stickies/Data/Library/Stickies/`
   - Parses `StickiesState.plist` for metadata (color, position, UUID)
   - Reads each `<UUID>.rtfd/` directory
   - Extracts `TXT.rtf` content + embedded images/attachments

2. **SQLite Database**
   - Single portable file in configurable location
   - Config file: `~/.config/sticky-situation/config.toml`
   - Default DB: `~/.local/share/sticky-situation/stickies.db`
   - Optional iCloud: `~/Library/Mobile Documents/com~apple~CloudDocs/stickies.db`

3. **Sync Engine**
   - Bidirectional merge between filesystem and database
   - Last-write-wins conflict resolution using modification timestamps
   - Logs conflicts for visibility

4. **CLI Interface**
   - `sticky sync` - run sync engine
   - `sticky new <text>` - create new sticky + reload app
   - `sticky search <query>` - full-text search with FTS5

## Database Schema

```sql
-- Core stickies table
CREATE TABLE stickies (
    uuid TEXT PRIMARY KEY,
    content_text TEXT,           -- plain text extraction for search
    rtf_data BLOB,                -- raw TXT.rtf file content
    plist_metadata BLOB,          -- serialized metadata from StickiesState.plist
    color TEXT,                   -- extracted for easy querying
    modified_at INTEGER,          -- Unix timestamp from filesystem
    created_at INTEGER,           -- Unix timestamp
    source_machine TEXT           -- hostname to track origin
);

-- Embedded files (images, attachments)
CREATE TABLE attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sticky_uuid TEXT,
    filename TEXT,                -- e.g., "IMG_001.tiff"
    content BLOB,                 -- raw file data
    FOREIGN KEY (sticky_uuid) REFERENCES stickies(uuid)
);

-- Full-text search index
CREATE VIRTUAL TABLE stickies_fts USING fts5(
    uuid,
    content_text,
    content=stickies,
    content_rowid=rowid
);
```

### Design Rationale

- Store raw RTF + plist as BLOBs to recreate exact RTFD bundles
- Extract plain text separately for efficient FTS5 search
- Track modification timestamps for conflict resolution
- Store source machine for debugging sync issues

## Sync Algorithm

### When `sticky sync` runs:

1. **Read filesystem state**
   - Parse `StickiesState.plist` for all sticky UUIDs + metadata
   - For each UUID, read `.rtfd/` directory (RTF + attachments)
   - Extract modification times from filesystem

2. **Read database state**
   - Load all UUIDs and `modified_at` timestamps from DB

3. **Three-way categorization:**
   - **New on filesystem** - UUID exists locally but not in DB → INSERT into DB
   - **New in database** - UUID exists in DB but not locally → CREATE .rtfd bundle on filesystem
   - **Modified on both** - UUID exists in both:
     - Compare `modified_at` timestamps
     - Last-write-wins: keep whichever is newer
     - Log conflict to stderr/file for visibility

4. **Write changes**
   - Update DB records for filesystem changes
   - Create/update .rtfd bundles for DB changes
   - Update `StickiesState.plist` with new entries
   - Send HUP signal to Stickies.app (or relaunch it)

### Edge Cases

- Stickies.app not running → skip reload step
- DB doesn't exist → create and do initial import
- Timestamps identical → skip (no changes)
- Corrupted .rtfd → log warning, skip, continue

## CLI Commands

### `sticky sync`

```bash
sticky sync [--dry-run] [--verbose]
```

- Runs the sync algorithm
- `--dry-run`: show what would change without making changes
- `--verbose`: log detailed sync operations
- Exits with error code if conflicts detected (but completes sync)

### `sticky new`

```bash
sticky new "My new sticky note"
sticky new  # opens $EDITOR for longer content
```

- Generates new UUID
- Creates `<UUID>.rtfd/TXT.rtf` with minimal RTF formatting
- Adds entry to `StickiesState.plist` (yellow, default position)
- Inserts into database
- Sends signal to Stickies.app to reload
- Outputs UUID for reference

### `sticky search`

```bash
sticky search "meeting notes"
sticky search --color yellow "todo"
```

- Uses FTS5 to search `content_text`
- Optional `--color` filter
- Returns: UUID, preview snippet, color
- Optional: `--open <uuid>` to focus sticky in Stickies.app

## Configuration

**File:** `~/.config/sticky-situation/config.toml`

```toml
[database]
path = "~/.local/share/sticky-situation/stickies.db"
# or: path = "~/Library/Mobile Documents/com~apple~CloudDocs/stickies.db"

[sync]
log_conflicts = true
conflict_log_path = "~/.local/share/sticky-situation/conflicts.log"
```

## Error Handling

### Filesystem Errors

- Stickies directory doesn't exist → suggest launching Stickies.app first
- Permission denied → explain sandboxing/permissions
- Corrupted .rtfd bundles → log warning, skip sticky, continue

### Database Errors

- DB locked → retry with exponential backoff
- Corrupted DB → offer to rebuild from filesystem
- Disk full → clear error message

### Stickies.app Communication

- App not running → skip reload, warn user
- HUP signal fails → fall back to full relaunch
- Can't parse plist → abort with helpful error

## Testing Strategy

### Unit Tests

- RTF parsing (minimal formatting)
- Plist parsing/writing
- Timestamp comparison logic
- UUID generation

### Integration Tests

- Test fixtures with sample .rtfd bundles
- Sync algorithm scenarios (new, modified, deleted)
- Database round-trip (write → read → verify)

### Manual Testing

- Test against real Stickies.app on macOS
- Verify sync across two test directories
- Ensure Stickies.app reload works correctly

## Implementation Notes

### Rust Dependencies

- `rusqlite` - SQLite with FTS5 support
- `plist` - Parse/write binary/XML plists
- `uuid` - Generate UUIDs for new stickies
- `clap` - CLI argument parsing
- `toml` - Configuration file parsing
- `directories` - XDG directory support

### RTF Handling

- Store raw RTF as-is (no parsing required for sync)
- Extract plain text using simple RTF stripping regex for search
- Future enhancement: proper RTF parsing if needed

### Stickies.app Reload

- Try sending `killall -HUP Stickies`
- Fall back to `killall Stickies && open -a Stickies` if HUP fails
- Only reload if app is currently running
