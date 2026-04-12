use crate::server::assert_service::{assert_initialized, AssertError};
use std::path::Path;

pub(super) fn check_initialized(project_path: &Path) -> Result<(), AssertError> {
    assert_initialized(project_path)
}
