// ABOUTME: Config command implementation
// ABOUTME: Shows or edits the configuration file path

use std::env;
use std::process::Command;
use sticky_situation::config::Config;
use sticky_situation::{Result, StickyError};

pub fn run(edit: bool) -> Result<()> {
    let config_path = Config::ensure_config_exists()?;

    if edit {
        // Edit mode: open in $EDITOR
        let editor = env::var("EDITOR").map_err(|_| {
            eprintln!("Error: EDITOR environment variable not set");
            eprintln!("Config file is at: {}", config_path.display());
            StickyError::Config("EDITOR environment variable not set".into())
        })?;

        let status = Command::new(editor)
            .arg(&config_path)
            .status()
            .map_err(|e| StickyError::Config(format!("Failed to start editor: {}", e)))?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
    } else {
        // Print mode: just print the path
        println!("{}", config_path.display());
    }

    Ok(())
}
