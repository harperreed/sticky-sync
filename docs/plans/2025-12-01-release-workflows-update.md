# Release Workflows Update

## Purpose

Update release.yml and homebrew.yml workflows from micropub to sticky-sync.

## Changes Required

### Binary Name

Update Cargo.toml:
```toml
[[bin]]
name = "sticky-sync"
path = "src/main.rs"
```

### release.yml Updates

Replace all instances:
- `micropub` → `sticky-sync` (binary and archive names)
- `harperreed/micropub` → `harperreed/sticky-sync` (repository references)

Archive names become:
- `sticky-sync-x86_64-apple-darwin.tar.gz`
- `sticky-sync-aarch64-apple-darwin.tar.gz`
- `sticky-sync-x86_64-unknown-linux-gnu.tar.gz`

Keep unchanged:
- Three platform targets (macOS x86_64, macOS aarch64, Linux x86_64)
- Crates.io publishing step
- Version detection logic
- Trigger conditions
- Archive structure (just the binary, no subdirectories)

### homebrew.yml Updates

Replace all instances:
- `micropub` → `sticky-sync` (binary, formula, bottles)
- `harperreed/micropub` → `harperreed/sticky-sync` (repository)
- `class Micropub` → `class StickySync` (formula class)
- `micropub.rb` → `sticky-sync.rb` (formula filename)

Update formula metadata:
- Description: "macOS Stickies synchronization tool"
- Homepage: "https://github.com/harperreed/sticky-sync"
- License: "MIT"

Update test command:
- `shell_output("#{bin}/sticky-sync --version")`

Bottle naming:
- `sticky-sync-${VERSION}.ventura.bottle.tar.gz`
- `sticky-sync-${VERSION}.arm64_sonoma.bottle.tar.gz`

Directory structure in bottles:
- `sticky-sync/${VERSION}/bin/sticky-sync`

Keep unchanged:
- Homebrew tap repository: `harperreed/homebrew-tap`
- Formula path: `Formula/sticky-sync.rb`
- Bottle building logic
- SHA256 validation
- Version detection and retry logic

## Implementation Steps

1. Update Cargo.toml binary name
2. Update release.yml with find/replace
3. Update homebrew.yml with find/replace
4. Verify binary builds with correct name
5. Commit all changes together

## Testing

- Build verification: `cargo build --release` produces `target/release/sticky-sync`
- Workflows test on next version bump
- Homebrew workflow supports manual trigger via workflow_dispatch

## Files Modified

- Cargo.toml
- .github/workflows/release.yml
- .github/workflows/homebrew.yml
