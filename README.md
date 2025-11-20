# sticky-situation

A Rust CLI tool to sync macOS Stickies across machines using a portable SQLite database.

## Features

- ✅ Bidirectional sync between Stickies.app and SQLite database
- ✅ Full-text search with FTS5
- ✅ List and show stickies from CLI
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

# After creating, sync to import the sticky with Stickies.app's UUID
sticky sync
```

**Note**: Stickies.app assigns its own UUID when importing new stickies, so you should run `sync` after creating to update your database with the correct UUID.

### Search stickies

```bash
sticky search "meeting notes"
sticky search --color yellow "todo"
```

### List all stickies

```bash
sticky list
sticky list --color yellow
```

### Show a specific sticky

```bash
sticky show <uuid>
```

### Reload Stickies.app

```bash
sticky hup
```

Sends a HUP signal to Stickies.app to reload it. Useful after manually editing files or syncing from another machine.

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

## Known Limitations

- **Window positioning**: When creating new stickies via `sticky new`, Stickies.app manages window positioning using its own internal logic. The CLI cannot control where new sticky windows appear on screen - they will be positioned by Stickies.app when it reloads.

- **UUID assignment**: Stickies.app assigns its own UUID to newly created stickies when it imports them. After running `sticky new`, you should run `sticky sync` to update your database with the UUID that Stickies.app assigned.

## Architecture

See [design document](docs/plans/2025-11-19-stickies-sync-design.md) for details.

## Testing

```bash
cargo test
```

## License

MIT
