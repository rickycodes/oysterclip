use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "clipboard-watcher",
    about = "Monitor clipboard activity and store history",
    long_about = "A clipboard monitoring tool that automatically captures and stores text and image clipboard entries to a local SQLite database.

Storage:
  • Text history: Stored in ~/.clipboard_history.db
    • Text content is encrypted using the OS keychain
  • Image history: Saved to clipboard_images/ directory
  • Configuration: ~/.clipboard-watcher.toml (TOML format)

Default behavior:
  When run without any subcommand, clipboard-watcher will start monitoring your clipboard and storing new entries.

Examples:
  • Start watching (default): clipboard-watcher
  • Show version: clipboard-watcher version
  • Show version (flag): clipboard-watcher --version",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        about = "Start watching the clipboard for changes",
        long_about = "Continuously monitors the system clipboard and saves new text and image entries to history.

This is the default behavior when no subcommand is provided.

History storage:
  • Maximum 500 entries by default (configurable via config file)
  • Older entries are automatically pruned when limit is reached
  • Text entries are encrypted using ChaCha20-Poly1305 with a key stored in your OS keychain

What gets captured:
  • Plain text: Any text copied to clipboard (URLs, code, notes, etc.)
  • Images: Screenshots or images copied to clipboard
  • Text types are automatically detected: URLs, JSON, multiline text, etc.

What gets skipped:
  • Empty text selections
  • Image data URIs (text representations of images)

Examples:
  clipboard-watcher watch
  clipboard-watcher        # Same as above"
    )]
    Watch,

    #[command(
        about = "Display version information",
        long_about = "Shows the current version of clipboard-watcher.

Alternative methods:
  • --version flag
  • -V short flag

Example:
  clipboard-watcher version
  clipboard-watcher --version
  clipboard-watcher -V"
    )]
    Version,
}
