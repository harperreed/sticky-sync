// ABOUTME: HUP command implementation
// ABOUTME: Sends HUP signal to Stickies.app to reload it

use sticky_situation::Result;
use std::process::Command;

fn reload_stickies_app() -> Result<()> {
    // Check if Stickies is running
    let output = Command::new("pgrep")
        .arg("Stickies")
        .output()?;

    if output.status.success() {
        println!("Sending HUP signal to Stickies.app...");

        // Try HUP first
        let hup_result = Command::new("killall")
            .arg("-HUP")
            .arg("Stickies")
            .status()?;

        if hup_result.success() {
            println!("✓ HUP signal sent successfully");
        } else {
            // Fall back to full restart
            println!("HUP failed, restarting Stickies.app...");
            Command::new("killall").arg("Stickies").status()?;
            std::thread::sleep(std::time::Duration::from_millis(500));
            Command::new("open").arg("-a").arg("Stickies").status()?;
            println!("✓ Stickies.app restarted");
        }
    } else {
        // Not running, just launch it
        println!("Stickies.app is not running, launching...");
        Command::new("open").arg("-a").arg("Stickies").status()?;
        println!("✓ Stickies.app launched");
    }

    Ok(())
}

pub fn run() -> Result<()> {
    reload_stickies_app()
}
