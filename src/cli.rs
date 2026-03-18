pub(crate) fn print_help() {
    println!(
        "\
{name} {version}

Usage:
  {name} [OPTIONS]

Options:
  -h, --help       Show this help message and exit
  -V, --version    Show version information and exit

Config:
  Uses {recipient_env} or .clipboard-watcher.toml for GPG recipient configuration.
",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        recipient_env = crate::common::GPG_RECIPIENT_ENV,
    );
}

pub(crate) fn print_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
