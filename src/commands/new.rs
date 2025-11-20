// ABOUTME: New command implementation
// ABOUTME: Creates new sticky in filesystem and database, then reloads Stickies.app

use std::path::PathBuf;
use std::process::Command;
use sticky_situation::{
    config::Config,
    database::{Database, Sticky},
    filesystem::rtfd::RtfdBundle,
    Result, StickyError,
};
use uuid::Uuid;

fn stickies_dir() -> Result<PathBuf> {
    let home =
        std::env::var("HOME").map_err(|_| StickyError::StickiesNotFound("HOME not set".into()))?;

    Ok(PathBuf::from(home).join("Library/Containers/com.apple.Stickies/Data/Library/Stickies"))
}

fn reload_stickies_app() -> Result<()> {
    // Check if Stickies is running
    let output = Command::new("pgrep").arg("Stickies").output()?;

    if output.status.success() {
        // Kill and restart for proper reload
        Command::new("killall").arg("Stickies").status()?;
        std::thread::sleep(std::time::Duration::from_millis(500));
        Command::new("open").arg("-a").arg("Stickies").status()?;
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
