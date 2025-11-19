// ABOUTME: New command implementation
// ABOUTME: Creates new sticky in filesystem and database, then reloads Stickies.app

use sticky_situation::{
    Result, StickyError,
    config::Config,
    database::{Database, Sticky},
    filesystem::{rtfd::RtfdBundle, plist},
};
use std::path::PathBuf;
use uuid::Uuid;
use std::process::Command;

fn stickies_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| StickyError::StickiesNotFound("HOME not set".into()))?;

    Ok(PathBuf::from(home)
        .join("Library/Containers/com.apple.Stickies/Data/Library/Stickies"))
}

fn reload_stickies_app() -> Result<()> {
    // Check if Stickies is running
    let output = Command::new("pgrep")
        .arg("Stickies")
        .output()?;

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

fn calculate_window_position(stickies_path: &PathBuf) -> (i32, i32) {
    let plist_path = stickies_path.join("StickiesState.plist");

    // Try to read existing positions
    if let Ok(metadata_map) = plist::read_stickies_state(&plist_path) {
        // Extract all window positions
        let mut positions: Vec<(i32, i32)> = Vec::new();

        for (_, metadata) in metadata_map.iter() {
            // Parse frame string like "{{100, 200}, {300, 400}}"
            if let Some(coords) = parse_frame(&metadata.frame) {
                positions.push(coords);
            }
        }

        // Find a non-overlapping position using cascade pattern
        if !positions.is_empty() {
            // Start at the last position and offset by 30 pixels
            if let Some(&(last_x, last_y)) = positions.last() {
                let new_x = last_x + 30;
                let new_y = last_y + 30;

                // Wrap around if we go too far right/down
                if new_x > 1200 || new_y > 800 {
                    return (100, 100);
                }

                return (new_x, new_y);
            }
        }
    }

    // Default position if plist doesn't exist or is empty
    (100, 100)
}

fn parse_frame(frame: &str) -> Option<(i32, i32)> {
    // Parse "{{x, y}, {width, height}}" format
    let parts: Vec<&str> = frame.split(',').collect();
    if parts.len() >= 2 {
        let x_str = parts[0].trim_start_matches('{').trim();
        let y_str = parts[1].split('}').next()?.trim();

        if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
            return Some((x, y));
        }
    }
    None
}

fn update_stickies_plist(stickies_path: &PathBuf, uuid: &str, x: i32, y: i32) -> Result<()> {
    use std::fs;
    use std::io::Write;

    let plist_path = stickies_path.join("StickiesState.plist");

    // Read existing content or start with empty plist
    let existing_content = if plist_path.exists() {
        fs::read_to_string(&plist_path).unwrap_or_default()
    } else {
        String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
            <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
            <plist version=\"1.0\">\n<dict>\n</dict>\n</plist>\n"
        )
    };

    // Create entry for new sticky
    let new_entry = format!(
        "\t<key>{}</key>\n\
        \t<dict>\n\
        \t\t<key>Color</key>\n\
        \t\t<integer>0</integer>\n\
        \t\t<key>Frame</key>\n\
        \t\t<string>{{{}, {}}}, {{250, 250}}</string>\n\
        \t\t<key>Floating</key>\n\
        \t\t<false/>\n\
        \t</dict>\n",
        uuid, x, y
    );

    // Insert before closing dict tag
    let updated_content = existing_content.replace("</dict>\n</plist>", &format!("{}</dict>\n</plist>", new_entry));

    // Write back
    let mut file = fs::File::create(&plist_path)?;
    file.write_all(updated_content.as_bytes())?;

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

    // Calculate non-overlapping window position and update plist
    let (x, y) = calculate_window_position(&stickies_path);
    update_stickies_plist(&stickies_path, &uuid, x, y)?;

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
