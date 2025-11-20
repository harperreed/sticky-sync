// ABOUTME: Search command implementation
// ABOUTME: Full-text search using SQLite FTS5 with optional color filtering

use sticky_situation::{config::Config, database::Database, Result};

pub fn run(query: &str, color: Option<&str>) -> Result<()> {
    let config = Config::load()?;
    let db = Database::create(&config.database_path)?;

    let results = db.search(query)?;

    let filtered: Vec<_> = results
        .iter()
        .filter(|s| color.is_none_or(|c| s.color == c))
        .collect();

    if filtered.is_empty() {
        println!("No results found for '{}'", query);
        return Ok(());
    }

    println!("Found {} result(s):\n", filtered.len());

    for sticky in filtered {
        println!("UUID: {}", sticky.uuid);
        println!("Color: {}", sticky.color);

        let preview = if sticky.content_text.len() > 100 {
            format!("{}...", &sticky.content_text[..100])
        } else {
            sticky.content_text.clone()
        };

        println!("Preview: {}", preview);
        println!("Modified: {}", sticky.modified_at);
        println!("---");
    }

    Ok(())
}
