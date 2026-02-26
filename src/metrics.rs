use std::time::Instant;
use tracing::info;

/// A timer that logs the duration of an operation when dropped.
///
/// Use this to measure and log how long operations take.
///
/// # Example
///
/// ```ignore
/// async fn create_issue(&self, request: Request<CreateIssueRequest>) -> Result<Response<CreateIssueResponse>, Status> {
///     let _timer = OperationTimer::new("create_issue");
///     // ... implementation ...
/// }
/// ```
pub struct OperationTimer {
    name: &'static str,
    start: Instant,
}

impl OperationTimer {
    /// Create a new timer for the given operation name.
    #[must_use]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        info!(
            operation = %self.name,
            duration_ms = %duration.as_millis(),
            "Operation completed"
        );
    }
}

/// Generate a short request ID for correlation.
#[must_use]
pub fn generate_request_id() -> String {
    let uuid_str = uuid::Uuid::new_v4().to_string();
    uuid_str.get(..8).unwrap_or(&uuid_str).to_string()
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
