// ABOUTME: Sync engine for bidirectional merge between filesystem and database
// ABOUTME: Implements last-write-wins strategy using modification timestamps

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum SyncAction {
    NewOnFilesystem(String),   // UUID needs to be inserted into DB
    NewInDatabase(String),      // UUID needs to be written to filesystem
    UpdateFilesystem(String),   // DB version is newer
    UpdateDatabase(String),     // Filesystem version is newer
    NoChange(String),           // Timestamps match
}

pub struct SyncEngine;

impl SyncEngine {
    pub fn categorize(
        fs_uuids: &[String],
        db_times: &HashMap<String, i64>,
        fs_times: &HashMap<String, i64>,
    ) -> Vec<SyncAction> {
        let mut actions = Vec::new();

        // Check filesystem UUIDs
        for uuid in fs_uuids {
            match db_times.get(uuid) {
                None => {
                    // New on filesystem
                    actions.push(SyncAction::NewOnFilesystem(uuid.clone()));
                }
                Some(&db_time) => {
                    let fs_time = fs_times.get(uuid).copied().unwrap_or(0);

                    if fs_time > db_time {
                        actions.push(SyncAction::UpdateDatabase(uuid.clone()));
                    } else if db_time > fs_time {
                        actions.push(SyncAction::UpdateFilesystem(uuid.clone()));
                    } else {
                        actions.push(SyncAction::NoChange(uuid.clone()));
                    }
                }
            }
        }

        // Check for UUIDs only in database
        for (uuid, _) in db_times {
            if !fs_uuids.contains(uuid) {
                actions.push(SyncAction::NewInDatabase(uuid.clone()));
            }
        }

        actions
    }
}
