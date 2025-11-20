# Config Command Design

## Purpose

Add `sticky config` command to show or edit the configuration file.

## CLI Interface

```rust
/// Show or edit configuration file
Config {
    /// Open config in $EDITOR
    #[arg(long, short)]
    edit: bool,
}
```

### Behavior

- `sticky config` prints the full path to config.toml
- `sticky config --edit` opens config in $EDITOR

### Examples

```bash
# Print config path for shell composition
sticky config
# Output: /Users/harper/Library/Application Support/sticky-situation/config.toml

# Use with other commands
cat $(sticky config)
ls -l $(sticky config)

# Edit in configured editor
EDITOR=vim sticky config --edit
```

## Implementation

### File Creation

When config file does not exist:

1. Create config directory using ProjectDirs
2. Generate Config::default()
3. Serialize to TOML
4. Write to config.toml

### TOML Format

```toml
# sticky-situation configuration

database_path = "/Users/harper/Library/Application Support/sticky-situation/stickies.db"
log_conflicts = true
conflict_log_path = "/Users/harper/Library/Application Support/sticky-situation/conflicts.log"
```

### Edit Mode Flow

1. Create config if it does not exist
2. Check $EDITOR environment variable
3. If not set: print error and path, exit 1
4. If set: spawn $EDITOR with config path, wait for exit
5. Return editor's exit code

### Print Mode Flow

1. Create config if it does not exist
2. Print path to stdout
3. Exit 0

## Error Handling

### Cannot determine config directory
- Return StickyError::Config
- Exit code 1

### Cannot create directory or file
- IO error propagates through ? operator
- Standard error messages for permissions and disk space

### Editor not set (edit mode only)
- Print: "Error: EDITOR environment variable not set"
- Print: "Config file is at: <path>"
- Exit code 1

### Editor fails to start
- IO error propagates
- Command not found handled by shell

### Editor exits non-zero
- Pass through editor's exit code
- Editor's error messages visible to user

## Testing

Unit tests in tests/config_test.rs:
- Config serializes to valid TOML
- Default config values are sensible
- Config path matches Config::load() logic

Manual integration testing:
- sticky config prints correct path
- sticky config --edit with EDITOR=cat shows content
- Config auto-creates on first run

## Files Modified

- src/commands/config.rs (new)
- src/main.rs (add Config variant, match arm)
- src/commands/mod.rs (add config module)
- tests/config_test.rs (new)
