// ABOUTME: CLI entry point for sticky-situation
// ABOUTME: Dispatches to sync, new, and search subcommands

use clap::{Parser, Subcommand};
use sticky_situation::Result;

mod commands;

#[derive(Parser)]
#[command(name = "sticky")]
#[command(about = "Sync macOS Stickies across machines", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync stickies between filesystem and database
    Sync {
        #[arg(long)]
        dry_run: bool,

        #[arg(long, short)]
        verbose: bool,
    },

    /// Create a new sticky note
    New {
        /// Text content of the sticky
        text: Option<String>,
    },

    /// Search stickies by content
    Search {
        /// Search query
        query: String,

        #[arg(long)]
        color: Option<String>,
    },

    /// List all stickies
    List {
        #[arg(long)]
        color: Option<String>,
    },

    /// Show a specific sticky by UUID
    Show {
        /// UUID of the sticky to display
        uuid: String,
    },

    /// Send HUP signal to reload Stickies.app
    Hup,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync { dry_run, verbose } => commands::sync::run(dry_run, verbose),
        Commands::New { text } => commands::new::run(text),
        Commands::Search { query, color } => commands::search::run(&query, color.as_deref()),
        Commands::List { color } => commands::list::run(color.as_deref()),
        Commands::Show { uuid } => commands::show::run(&uuid),
        Commands::Hup => commands::hup::run(),
    }
}
