pub(crate) fn print_help() {
    println!(
        "\
{name} {version}

Usage:
  {name} [OPTIONS]

Options:
  -h, --help       Show this help message and exit
  -V, --version    Show version information and exit

Storage:
  Writes clipboard history to .clipboard_history.db and encrypts text content using the OS keychain.
",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
    );
}

pub(crate) fn print_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
