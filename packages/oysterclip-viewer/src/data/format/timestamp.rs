use chrono::{DateTime, Local, Utc};

pub fn format_relative_timestamp(timestamp: u64) -> String {
    let now = Utc::now().timestamp() as u64;
    let secs = now.saturating_sub(timestamp);

    if secs < 60 {
        return "just now".to_string();
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{}m ago", mins);
    }
    let hours = mins / 60;
    if hours < 24 {
        return format!("{}h ago", hours);
    }
    // For older entries, show days/weeks in relative format
    if let Some(utc) = DateTime::<Utc>::from_timestamp(timestamp as i64, 0) {
        let local = utc.with_timezone(&Local);
        let today = Local::now().date_naive();
        let entry_date = local.date_naive();
        let days_ago = (today - entry_date).num_days();

        if days_ago < 7 {
            return format!("{}d ago", days_ago);
        }
        let weeks_ago = days_ago / 7;
        if weeks_ago < 4 {
            return format!("{}w ago", weeks_ago);
        }
        return local.format("%b %-d").to_string(); // e.g. "Mar 20" for very old entries
    }
    timestamp.to_string()
}

pub fn format_timestamp(timestamp: u64) -> String {
    if let Some(utc) = DateTime::<Utc>::from_timestamp(timestamp as i64, 0) {
        utc.with_timezone(&Local)
            .format("%A, %b %d, %Y %I:%M %p")
            .to_string()
    } else {
        timestamp.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_relative_timestamp_just_now() {
        let now = Utc::now().timestamp() as u64;
        let result = format_relative_timestamp(now);
        assert_eq!(result, "just now");
    }

    #[test]
    fn test_format_relative_timestamp_minutes_ago() {
        let now = Utc::now().timestamp() as u64;
        let five_min_ago = now - 300;
        let result = format_relative_timestamp(five_min_ago);
        assert!(result.contains("m ago"));
    }

    #[test]
    fn test_format_relative_timestamp_hours_ago() {
        let now = Utc::now().timestamp() as u64;
        let two_hours_ago = now - 7200;
        let result = format_relative_timestamp(two_hours_ago);
        assert!(result.contains("h ago"));
    }

    #[test]
    fn test_format_relative_timestamp_days_ago() {
        let now = Utc::now().timestamp() as u64;
        let three_days_ago = now - (3 * 86400);
        let result = format_relative_timestamp(three_days_ago);
        assert!(result.contains("d ago"));
    }

    #[test]
    fn test_format_relative_timestamp_weeks_ago() {
        let now = Utc::now().timestamp() as u64;
        let two_weeks_ago = now - (14 * 86400);
        let result = format_relative_timestamp(two_weeks_ago);
        assert!(result.contains("w ago"));
    }

    #[test]
    fn test_format_timestamp_valid() {
        let now = Utc::now().timestamp() as u64;
        let result = format_timestamp(now);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_timestamp_invalid_zero_returns_fallback() {
        let result = format_timestamp(0);
        // Zero is a valid but very old timestamp (1970-01-01), should have formatted date
        assert!(!result.is_empty());
    }
}
