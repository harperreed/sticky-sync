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
        println!("Restarting Stickies.app...");

        // Kill Stickies.app
        Command::new("killall").arg("Stickies").status()?;

        // Wait for it to fully quit
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Relaunch it
        Command::new("open").arg("-a").arg("Stickies").status()?;

        println!("✓ Stickies.app restarted");
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
