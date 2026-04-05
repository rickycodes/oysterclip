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
