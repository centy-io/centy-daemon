use crate::config::item_type_config::BUILTIN_ITEM_TYPE_FOLDERS;
use crate::server::proto::CreateItemTypeRequest;

/// Validate the request fields. Returns an error (code, message) on failure.
pub(super) fn validate_request(req: &CreateItemTypeRequest) -> Result<(), (String, String)> {
    if req.name.trim().is_empty() {
        return Err(("VALIDATION_ERROR".into(), "name must not be empty".into()));
    }
    // Built-in folders bypass the slug rule (`user_stories` is not a slug).
    let is_builtin = BUILTIN_ITEM_TYPE_FOLDERS.contains(&req.plural.as_str());
    if req.plural.is_empty() || (!is_builtin && slug::slugify(&req.plural) != req.plural) {
        return Err((
            "VALIDATION_ERROR".into(),
            "plural must be lowercase alphanumeric with hyphens (e.g., \"bugs\", \"epics\")".into(),
        ));
    }
    if req.identifier != "uuid" && req.identifier != "slug" {
        return Err((
            "VALIDATION_ERROR".into(),
            "identifier must be \"uuid\" or \"slug\"".into(),
        ));
    }
    Ok(())
}
