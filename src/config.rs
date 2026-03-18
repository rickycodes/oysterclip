use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;

use crate::common::{CONFIG_FILE, GPG_RECIPIENT_ENV};

#[derive(Deserialize)]
struct AppConfig {
    gpg_recipient: Option<String>,
}

fn io_error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

fn normalize_config_value(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn parse_gpg_recipient_config(contents: &str) -> io::Result<Option<String>> {
    let config: AppConfig = toml::from_str(contents)
        .map_err(|err| io_error(format!("failed to parse {}: {err}", CONFIG_FILE)))?;
    Ok(normalize_config_value(config.gpg_recipient))
}

pub(crate) fn resolve_gpg_recipient() -> io::Result<String> {
    if let Some(recipient) = normalize_config_value(std::env::var(GPG_RECIPIENT_ENV).ok()) {
        return Ok(recipient);
    }

    let config_path = Path::new(CONFIG_FILE);
    if config_path.exists() {
        let contents = fs::read_to_string(config_path)?;
        if let Some(recipient) = parse_gpg_recipient_config(&contents)? {
            return Ok(recipient);
        }
    }

    Err(io_error(format!(
        "missing GPG recipient; set {} or add gpg_recipient = \"your-key-id-or-email\" to {}",
        GPG_RECIPIENT_ENV, CONFIG_FILE
    )))
}

#[cfg(test)]
mod tests {
    use super::parse_gpg_recipient_config;

    #[test]
    fn parse_gpg_recipient_config_reads_value() {
        let config = "gpg_recipient = \"ricky@example.com\"\n";
        assert_eq!(
            parse_gpg_recipient_config(config).unwrap().as_deref(),
            Some("ricky@example.com")
        );
    }

    #[test]
    fn parse_gpg_recipient_config_treats_blank_as_missing() {
        let config = "gpg_recipient = \"   \"\n";
        assert_eq!(parse_gpg_recipient_config(config).unwrap(), None);
    }
}
