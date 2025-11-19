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
