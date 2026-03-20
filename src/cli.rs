use crate::constants::{
    HELP_FLAG_LONG, HELP_FLAG_SHORT, HISTORY_FILE, VERSION_FLAG_LONG, VERSION_FLAG_SHORT,
};

pub(crate) fn print_help() {
    println!(
        "\
{name} {version}

Usage:
  {name} [OPTIONS]

Options:
  {help_short}, {help_long}       Show this help message and exit
  {version_short}, {version_long}    Show version information and exit

Storage:
  Writes clipboard history to {history_file} and encrypts text content using the OS keychain.
",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        help_short = HELP_FLAG_SHORT,
        help_long = HELP_FLAG_LONG,
        version_short = VERSION_FLAG_SHORT,
        version_long = VERSION_FLAG_LONG,
        history_file = HISTORY_FILE,
    );
}

pub(crate) fn print_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
