use chrono::{Datelike, Local, TimeZone, Utc};
use common::{ENTRY_TYPE_IMAGE, ENTRY_TYPE_TEXT};

use crate::data::entry::ClipboardEntry;

#[derive(Debug, Clone)]
struct QueryFilter {
    key: String,
    value: String,
}

/// Check if an entry matches the given query string.
///
/// Supports filter syntax: `type:text kind:password since:1h search text`
pub fn matches_query(entry: &ClipboardEntry, query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Parse filters from query (e.g., "type:image kind:url search text")
    let (filters, search_text) = parse_query_filters(trimmed);

    // Check type and kind filters first
    if !filters.is_empty() && !apply_filters(entry, &filters) {
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

fn apply_filters(entry: &ClipboardEntry, filters: &[QueryFilter]) -> bool {
    filters.iter().all(|filter| match filter.key.as_str() {
        "type" => apply_type_filter(entry, filter),
        "kind" => apply_kind_filter(entry, filter),
        "since" => apply_since_filter(entry, filter),
        _ => true,
    })
}

fn apply_type_filter(entry: &ClipboardEntry, filter: &QueryFilter) -> bool {
    match entry {
        ClipboardEntry::Text { .. } => {
            filter.value == ENTRY_TYPE_TEXT || filter.value == "pass" || filter.value == "password"
        }
        ClipboardEntry::Image { .. } => filter.value == ENTRY_TYPE_IMAGE,
    }
}

fn apply_kind_filter(entry: &ClipboardEntry, filter: &QueryFilter) -> bool {
    match entry {
        ClipboardEntry::Text { kind, content, .. } => {
            let entry_kind = if is_password(content) {
                "password"
            } else if let Some(k) = kind {
                k.as_str()
            } else {
                "text"
            };
            entry_kind.to_lowercase().contains(&filter.value)
        }
        ClipboardEntry::Image { .. } => false,
    }
}

fn apply_since_filter(entry: &ClipboardEntry, filter: &QueryFilter) -> bool {
    match parse_since_cutoff(&filter.value) {
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

    if value == "today" {
        let local = Local::now();
        let start = Local
            .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
            .single()?;
        return Some(start.with_timezone(&Utc).timestamp() as u64);
    }

    if value == "yesterday" {
        let local = Local::now() - chrono::Duration::days(1);
        let start = Local
            .with_ymd_and_hms(local.year(), local.month(), local.day(), 0, 0, 0)
            .single()?;
        return Some(start.with_timezone(&Utc).timestamp() as u64);
    }

    if let Some(n) = value.strip_suffix('m') {
        let minutes: i64 = n.parse().ok()?;
        return Some((now - chrono::Duration::minutes(minutes)).timestamp() as u64);
    }

    if let Some(n) = value.strip_suffix('h') {
        let hours: i64 = n.parse().ok()?;
        return Some((now - chrono::Duration::hours(hours)).timestamp() as u64);
    }

    if let Some(n) = value.strip_suffix('d') {
        let days: i64 = n.parse().ok()?;
        return Some((now - chrono::Duration::days(days)).timestamp() as u64);
    }

    if let Some(n) = value.strip_suffix('w') {
        let weeks: i64 = n.parse().ok()?;
        return Some((now - chrono::Duration::weeks(weeks)).timestamp() as u64);
    }

    None
}

fn is_password(content: &str) -> bool {
    crate::data::format::is_password(content)
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
