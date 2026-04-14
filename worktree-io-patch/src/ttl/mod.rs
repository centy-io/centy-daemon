use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// Workspace record and registry persistence.
pub mod registry;
pub use registry::{WorkspaceRecord, WorkspaceRegistry};

/// A time-to-live duration controlling how long a workspace remains active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ttl(#[serde(with = "humantime_serde")] Duration);

impl Ttl {
    /// Create a new [`Ttl`] wrapping the given duration.
    #[must_use]
    pub const fn new(duration: Duration) -> Self {
        Self(duration)
    }

    /// Return the inner [`Duration`].
    #[must_use]
    pub const fn duration(self) -> Duration {
        self.0
    }
}

impl std::fmt::Display for Ttl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", humantime::format_duration(self.0))
    }
}

impl std::str::FromStr for Ttl {
    type Err = humantime::DurationError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        humantime::parse_duration(s).map(Self)
    }
}

/// Returns `true` if the workspace has exceeded its TTL at the given instant.
///
/// Returns `false` when `created_at` is in the future relative to `now`.
#[must_use]
pub fn is_expired(record: &WorkspaceRecord, ttl: &Ttl, now: SystemTime) -> bool {
    now.duration_since(record.created_at)
        .map(|age| age >= ttl.0)
        .unwrap_or(false)
}

/// Returns workspaces that are both present on disk and have exceeded the TTL.
///
/// Entries whose [`WorkspaceRecord::path`] no longer exists are silently
/// skipped, making the registry self-healing on the next prune call.
#[must_use]
pub fn prune<'a>(
    records: &'a [WorkspaceRecord],
    ttl: &Ttl,
    now: SystemTime,
) -> Vec<&'a WorkspaceRecord> {
    records
        .iter()
        .filter(|r| r.path.exists() && is_expired(r, ttl, now))
        .collect()
}

#[cfg(test)]
#[path = "ttl_tests.rs"]
mod ttl_tests;

#[cfg(test)]
#[path = "prune_tests.rs"]
mod prune_tests;

#[cfg(test)]
#[path = "serde_tests.rs"]
mod serde_tests;
