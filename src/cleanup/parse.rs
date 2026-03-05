use chrono::Duration;

/// Parse a duration string like `"30d"`, `"24h"`, `"7d"` into a [`Duration`].
///
/// Returns `None` when the string is `"0"`, empty, or invalid.
/// Returns `Some(Duration)` with positive value for valid strings.
pub fn parse_retention_duration(s: &str) -> Option<Duration> {
    let s = s.trim();
    if s.is_empty() || s == "0" {
        return None;
    }
    if let Some(days) = s.strip_suffix('d') {
        let n: i64 = days.trim().parse().ok()?;
        if n <= 0 {
            return None;
        }
        return Some(Duration::days(n));
    }
    if let Some(hours) = s.strip_suffix('h') {
        let n: i64 = hours.trim().parse().ok()?;
        if n <= 0 {
            return None;
        }
        return Some(Duration::hours(n));
    }
    None
}
