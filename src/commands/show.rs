// ABOUTME: Show command implementation
// ABOUTME: Displays the full content of a specific sticky by UUID

use sticky_situation::{
    Result, StickyError,
    config::Config,
    database::Database,
};

pub fn run(uuid: &str) -> Result<()> {
    let config = Config::load()?;
    let db = Database::create(&config.database_path)?;

    match db.get_sticky(uuid)? {
        Some(sticky) => {
            println!("UUID: {}", sticky.uuid);
            println!("Color: {}", sticky.color);
            println!("Source Machine: {}", sticky.source_machine);
            println!("Created: {}", sticky.created_at);
            println!("Modified: {}", sticky.modified_at);
            println!("\nContent:");
            println!("{}", sticky.content_text);

            if !sticky.rtf_data.is_empty() {
                println!("\n(RTF data: {} bytes)", sticky.rtf_data.len());
            }

            Ok(())
        }
        None => {
            Err(StickyError::Config(format!("Sticky with UUID {} not found", uuid)))
        }
    }
}
