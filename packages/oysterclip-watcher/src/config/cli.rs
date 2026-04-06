use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "clipboard-watcher",
    about = "Monitor clipboard activity and store history",
    long_about = "A clipboard monitoring tool that automatically captures and stores text and image clipboard entries to a local SQLite database.

Storage:
  - Default base directory: the per-user app data directory for oysterclip
  - Text history: stored in .clipboard_history.db
    - Text content is encrypted using the OS keychain
  - Image history: optionally exported to clipboard_images/
  - Configuration: .clipboard-watcher.toml
  - Unix control socket: .clipboard-watcher.sock on unix targets

Default behavior:
  When run without any subcommand, clipboard-watcher will start monitoring your clipboard and storing new entries.

Examples:
  - Start watching (default): clipboard-watcher
  - Start paused: clipboard-watcher watch --paused
  - Pause a running watcher: clipboard-watcher control pause
  - Check watcher status: clipboard-watcher control status
  - Show version: clipboard-watcher version",
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
  - Maximum 500 entries by default (configurable via config file)
  - Older entries are automatically pruned when limit is reached
  - Text entries are encrypted using XChaCha20-Poly1305 with a key stored in your OS keychain
  - On unix targets, the running watcher also exposes a local control socket for pause/resume/status commands

What gets captured:
  - Plain text: any text copied to clipboard (URLs, code, notes, etc.)
  - Images: screenshots or images copied to clipboard
  - Text types are automatically detected: URLs, JSON, multiline text, etc.

What gets skipped:
  - Empty text selections
  - Image data URIs (text representations of images)

Examples:
  clipboard-watcher watch
  clipboard-watcher watch --paused
  clipboard-watcher        # Same as above"
    )]
    Watch(WatchArgs),

    #[command(about = "Control a running watcher via the local control socket")]
    Control(ControlCommand),

    #[command(
        about = "Display version information",
        long_about = "Shows the current version of clipboard-watcher.

Alternative methods:
  - --version flag
  - -V short flag

Example:
  clipboard-watcher version
  clipboard-watcher --version
  clipboard-watcher -V"
    )]
    Version,
}

#[derive(Args)]
pub struct WatchArgs {
    #[arg(long, help = "Start the watcher with capture paused")]
    pub paused: bool,
}

#[derive(Subcommand)]
pub enum ControlAction {
    #[command(about = "Pause a running watcher")]
    Pause,
    #[command(about = "Resume a running watcher")]
    Resume,
    #[command(about = "Show running watcher status")]
    Status,
}

#[derive(Args)]
pub struct ControlCommand {
    #[command(subcommand)]
    pub action: ControlAction,
}
