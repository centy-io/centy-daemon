use super::super::priority::{default_priority, validate_priority};
use super::types::IssueError;

pub fn resolve_priority(
    priority_opt: Option<u32>,
    config: Option<&crate::config::CentyConfig>,
    priority_levels: u32,
) -> Result<u32, IssueError> {
    match priority_opt {
        Some(p) => {
            validate_priority(p, priority_levels)?;
            Ok(p)
        }
        None => Ok(config
            .and_then(|c| c.defaults.get("priority"))
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or_else(|| default_priority(priority_levels))),
    }
}


