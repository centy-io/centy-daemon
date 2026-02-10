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
mod tests {
    use super::*;

    #[test]
    fn test_operation_timer_creation() {
        let timer = OperationTimer::new("test_op");
        assert_eq!(timer.name, "test_op");
    }

    #[test]
    fn test_operation_timer_drop_logs() {
        // Just verify it doesn't panic when dropped
        let _timer = OperationTimer::new("test_drop");
        // Timer will be dropped at end of scope
    }

    #[test]
    fn test_generate_request_id_format() {
        let id = generate_request_id();
        assert_eq!(id.len(), 8);
    }

    #[test]
    fn test_generate_request_id_unique() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_request_id_hex_chars() {
        let id = generate_request_id();
        // cspell:ignore hexdigit
        assert!(id.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
    }
}
