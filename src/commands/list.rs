// ABOUTME: List command implementation
// ABOUTME: Lists all stickies with optional color filtering

use sticky_situation::{config::Config, database::Database, Result};

pub fn run(color: Option<&str>) -> Result<()> {
    let config = Config::load()?;
    let db = Database::create(&config.database_path)?;

    // Use efficient single query instead of N+1 pattern
    let stickies = match color {
        Some(c) => db.get_stickies_by_color(c)?,
        None => db.get_all_stickies()?,
    };

    if stickies.is_empty() {
        if let Some(c) = color {
            println!("No {} stickies found", c);
        } else {
            println!("No stickies found");
        }
        return Ok(());
    }

    println!("Found {} sticky/stickies:\n", stickies.len());

    for sticky in stickies {
        let preview = if sticky.content_text.len() > 60 {
            format!("{}...", &sticky.content_text[..60])
        } else {
            sticky.content_text.clone()
        };

        println!("UUID: {}", sticky.uuid);
        println!("Color: {}", sticky.color);
        println!("Preview: {}", preview);
        println!("Modified: {}", sticky.modified_at);
        println!("---");
    }

    Ok(())
}
