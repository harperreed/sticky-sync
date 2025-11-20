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

    // Read filesystem state from .SavedStickiesState (the actual file Stickies.app uses)
    let plist_path = stickies_path.join(".SavedStickiesState");
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
