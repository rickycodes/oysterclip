use chrono::{Datelike, Local, TimeZone, Utc};
use common::{classification::is_password_with_config, ENTRY_TYPE_IMAGE, ENTRY_TYPE_TEXT};

use crate::config::settings::PasswordConfig;
use crate::data::entry::ClipboardEntry;

#[derive(Debug, Clone)]
struct QueryFilter {
    key: String,
    value: String,
}

/// Check if an entry matches the given query string.
///
/// Supports filter syntax: `type:text kind:password since:1h search text`
pub fn matches_query(
    entry: &ClipboardEntry,
    query: &str,
    password_config: &PasswordConfig,
) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Parse filters from query (e.g., "type:image kind:url search text")
    let (filters, search_text) = parse_query_filters(trimmed);

    // Check type and kind filters first
    if !filters.is_empty() && !apply_filters(entry, &filters, password_config) {
        return false;
    }

    // If no search text remains, we're done (filters alone matched)
    if search_text.is_empty() {
        return true;
    }

    // Apply text search on content and kind
    let search = search_text.to_lowercase();
    match entry {
        ClipboardEntry::Text { content, kind, .. } => {
            content.to_lowercase().contains(&search)
                || kind
                    .as_deref()
                    .map(|kind| kind.to_lowercase().contains(&search))
                    .unwrap_or(false)
        }
        ClipboardEntry::Image { path, .. } => path
            .as_deref()
            .map(|value| value.to_lowercase().contains(&search))
            .unwrap_or(false),
    }
}

fn parse_query_filters(query: &str) -> (Vec<QueryFilter>, String) {
    let mut filters = Vec::new();
    let mut search_parts = Vec::new();

    for part in query.split_whitespace() {
        if let Some((key, value)) = part.split_once(':') {
            if matches!(key, "type" | "kind" | "since") && !value.is_empty() {
                filters.push(QueryFilter {
                    key: key.to_lowercase(),
                    value: value.to_lowercase(),
                });
            } else {
                search_parts.push(part);
            }
        } else {
            search_parts.push(part);
        }
    }

    (filters, search_parts.join(" "))
}

fn apply_filters(
    entry: &ClipboardEntry,
    filters: &[QueryFilter],
    password_config: &PasswordConfig,
) -> bool {
    filters.iter().all(|filter| match filter.key.as_str() {
        "type" => matches_type_filter(entry, &filter.value),
        "kind" => matches_kind_filter(entry, &filter.value, password_config),
        "since" => matches_since_filter(entry, &filter.value),
        _ => true,
    })
}

fn matches_type_filter(entry: &ClipboardEntry, filter_value: &str) -> bool {
    match entry {
        ClipboardEntry::Text { .. } => {
            filter_value == ENTRY_TYPE_TEXT || filter_value == "pass" || filter_value == "password"
        }
        ClipboardEntry::Image { .. } => filter_value == ENTRY_TYPE_IMAGE,
    }
}

fn matches_kind_filter(
    entry: &ClipboardEntry,
    filter_value: &str,
    password_config: &PasswordConfig,
) -> bool {
    match entry {
        ClipboardEntry::Text { kind, content, .. } => {
            let entry_kind = if is_password_with_config(
                content,
                password_config.len,
                password_config.score_threshold,
            ) {
                "password"
            } else if let Some(k) = kind {
                k.as_str()
            } else {
                "text"
            };
            entry_kind.to_lowercase().contains(filter_value)
        }
        ClipboardEntry::Image { .. } => false,
    }
}

fn matches_since_filter(entry: &ClipboardEntry, filter_value: &str) -> bool {
    match parse_since_cutoff(filter_value) {
        Some(cutoff) => {
            let ts = match entry {
                ClipboardEntry::Text { timestamp, .. }
                | ClipboardEntry::Image { timestamp, .. } => *timestamp,
            };
            ts >= cutoff
        }
        None => true,
    }
}

/// Parse a `since:` value into a UTC unix timestamp cutoff.
/// Supported: `Nm` (N minutes), `Nh` (N hours), `Nd` (N days), `Nw` (N weeks), `today`, `yesterday`.
fn parse_since_cutoff(value: &str) -> Option<u64> {
    let now = Utc::now();

    // Try relative date shortcuts first
    if let Some(cutoff) = parse_relative_date(value) {
        return Some(cutoff);
    }

    // Try duration formats (e.g., 30m, 2h, 7d, 2w)
    parse_duration_offset(value, now)
}

fn parse_relative_date(value: &str) -> Option<u64> {
    match value {
        "today" => {
            let local = Local::now();
            let start = Local
                .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
                .single()?;
            Some(start.with_timezone(&Utc).timestamp() as u64)
        }
        "yesterday" => {
            let local = Local::now() - chrono::Duration::days(1);
            let start = Local
                .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
                .single()?;
            Some(start.with_timezone(&Utc).timestamp() as u64)
        }
        _ => None,
    }
}

fn parse_duration_offset(value: &str, now: chrono::DateTime<Utc>) -> Option<u64> {
    let (n, suffix) = extract_duration_parts(value)?;
    let cutoff = match suffix {
        'm' => now - chrono::Duration::minutes(n),
        'h' => now - chrono::Duration::hours(n),
        'd' => now - chrono::Duration::days(n),
        'w' => now - chrono::Duration::weeks(n),
        _ => return None,
    };
    Some(cutoff.timestamp() as u64)
}

fn extract_duration_parts(value: &str) -> Option<(i64, char)> {
    if value.is_empty() {
        return None;
    }
    let last_char = value.chars().last()?;
    if matches!(last_char, 'm' | 'h' | 'd' | 'w') {
        let num_str = &value[..value.len() - 1];
        let n = num_str.parse::<i64>().ok()?;
        Some((n, last_char))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_since_minutes() {
        let cutoff = parse_since_cutoff("30m");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::minutes(30)).timestamp() as u64;
        // Allow 1 second of variance due to test execution time
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }

    #[test]
    fn test_parse_since_hours() {
        let cutoff = parse_since_cutoff("2h");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::hours(2)).timestamp() as u64;
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }

    #[test]
    fn test_parse_since_days() {
        let cutoff = parse_since_cutoff("7d");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::days(7)).timestamp() as u64;
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }

    #[test]
    fn test_parse_since_weeks() {
        let cutoff = parse_since_cutoff("2w");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::weeks(2)).timestamp() as u64;
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }

    #[test]
    fn test_parse_since_today() {
        let cutoff = parse_since_cutoff("today");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let local = Local::now();
        let start = Local
            .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
            .single()
            .unwrap();
        let expected = start.with_timezone(&Utc).timestamp() as u64;
        assert_eq!(cutoff, expected);
    }

    #[test]
    fn test_parse_since_yesterday() {
        let cutoff = parse_since_cutoff("yesterday");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let local = Local::now() - chrono::Duration::days(1);
        let start = Local
            .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
            .single()
            .unwrap();
        let expected = start.with_timezone(&Utc).timestamp() as u64;
        assert_eq!(cutoff, expected);
    }

    #[test]
    fn test_parse_since_invalid() {
        assert!(parse_since_cutoff("invalid").is_none());
        assert!(parse_since_cutoff("abc").is_none());
        assert!(parse_since_cutoff("").is_none());
    }

    #[test]
    fn test_parse_since_one_minute() {
        let cutoff = parse_since_cutoff("1m");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::minutes(1)).timestamp() as u64;
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }

    #[test]
    fn test_parse_since_one_week() {
        let cutoff = parse_since_cutoff("1w");
        assert!(cutoff.is_some());
        let cutoff = cutoff.unwrap();
        let expected = (Utc::now() - chrono::Duration::weeks(1)).timestamp() as u64;
        assert!((cutoff as i64 - expected as i64).abs() <= 1);
    }
}
