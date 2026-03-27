use clap::Parser;
use std::sync::OnceLock;

static ARGS: OnceLock<Args> = OnceLock::new();

#[derive(Parser, Debug)]
#[command(about = "Clipboard history viewer")]
pub struct Args {
    /// Override theme for this session without changing the saved preference (dark|light)
    #[arg(long, value_name = "THEME", value_parser = ["dark", "light"])]
    pub theme: Option<String>,

    /// Path to the clipboard history database (overrides $CLIPBOARD_HISTORY_DB and the default location)
    #[arg(long, value_name = "PATH")]
    pub db: Option<String>,
}

/// Parse and cache CLI args. Call once at startup before launching the app.
pub fn parse() {
    ARGS.get_or_init(Args::parse);
}

pub fn args() -> &'static Args {
    ARGS.get_or_init(Args::parse)
}
