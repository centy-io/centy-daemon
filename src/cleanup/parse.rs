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
    let std_duration = humantime::parse_duration(s).ok()?;
    let chrono_duration = Duration::from_std(std_duration).ok()?;
    if chrono_duration <= Duration::zero() {
        return None;
    }
    Some(chrono_duration)
}
