use crate::server::proto::CreateItemTypeRequest;
/// Validate the plural field: must be lowercase alphanumeric + hyphens, non-empty.
pub(super) fn is_valid_plural(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}
/// Validate the request fields. Returns an error (code, message) on failure.
pub(super) fn validate_request(req: &CreateItemTypeRequest) -> Result<(), (String, String)> {
    if req.name.trim().is_empty() {
        return Err(("VALIDATION_ERROR".into(), "name must not be empty".into()));
    }
    if !is_valid_plural(&req.plural) {
        return Err((
            "VALIDATION_ERROR".into(),
            "plural must be lowercase alphanumeric with hyphens (e.g., \"bugs\", \"epics\")".into(),
        ));
    }
    if req.identifier != "uuid" && req.identifier != "slug" {
        return Err(("VALIDATION_ERROR".into(), "identifier must be \"uuid\" or \"slug\"".into()));
    }
    if !req.default_status.is_empty() {
        if req.statuses.is_empty() {
            return Err(("VALIDATION_ERROR".into(),
                "default_status provided but statuses list is empty".into()));
        }
        if !req.statuses.contains(&req.default_status) {
            return Err(("VALIDATION_ERROR".into(), format!(
                "default_status \"{}\" must be in statuses list {:?}",
                req.default_status, req.statuses
            )));
        }
    }
    Ok(())
}
