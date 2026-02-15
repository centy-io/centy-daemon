use super::actions::make_action;
use super::proto::{ActionCategory, EntityAction};

/// Build doc-specific actions.
pub fn build_doc_actions(has_entity_id: bool) -> Vec<EntityAction> {
    let mut actions = vec![make_action(
        "create",
        "Create Doc",
        ActionCategory::Crud as i32,
        "c",
        false,
    )];
    if has_entity_id {
        actions.extend([
            make_action("delete", "Delete", ActionCategory::Crud as i32, "d", true),
            make_action(
                "duplicate",
                "Duplicate",
                ActionCategory::Crud as i32,
                "D",
                false,
            ),
            make_action(
                "move",
                "Move to Project",
                ActionCategory::Crud as i32,
                "m",
                false,
            ),
        ]);
    }
    actions
}
