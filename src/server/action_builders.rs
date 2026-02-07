use super::actions::{make_action, make_status_action};
use super::proto::{ActionCategory, EntityAction};

/// Build issue-specific actions.
pub fn build_issue_actions(
    entity_status: Option<&String>,
    allowed_states: &[String],
    vscode_available: bool,
    terminal_available: bool,
    has_entity_id: bool,
) -> Vec<EntityAction> {
    let mut actions = vec![make_action(
        "create",
        "Create Issue",
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
            make_action("mode:plan", "Plan", ActionCategory::Mode as i32, "p", false),
            make_action(
                "mode:implement",
                "Implement",
                ActionCategory::Mode as i32,
                "i",
                false,
            ),
            make_action(
                "mode:deepdive",
                "Deep Dive",
                ActionCategory::Mode as i32,
                "D",
                false,
            ),
        ]);
        for state in allowed_states {
            actions.push(make_status_action(state, entity_status, false));
        }
        actions.push(EntityAction {
            id: "open_in_vscode".to_string(),
            label: "Open in VSCode".to_string(),
            category: ActionCategory::External as i32,
            enabled: vscode_available,
            disabled_reason: if vscode_available {
                String::new()
            } else {
                "VSCode not available".to_string()
            },
            destructive: false,
            keyboard_shortcut: "o".to_string(),
        });
        actions.push(EntityAction {
            id: "open_in_terminal".to_string(),
            label: "Open in Terminal".to_string(),
            category: ActionCategory::External as i32,
            enabled: terminal_available,
            disabled_reason: if terminal_available {
                String::new()
            } else {
                "Terminal not available".to_string()
            },
            destructive: false,
            keyboard_shortcut: "t".to_string(),
        });
    }
    actions
}
