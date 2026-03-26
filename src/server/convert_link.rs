use crate::link::{LinkView, TargetType};

use super::proto::{Link as ProtoLink, LinkTargetType};

pub fn proto_link_target_to_internal(proto_type: LinkTargetType) -> TargetType {
    match proto_type {
        LinkTargetType::Issue | LinkTargetType::Unspecified => TargetType::issue(),
        LinkTargetType::Doc => TargetType::new("doc"),
    }
}

/// Resolve the target type from either a string field (preferred) or the legacy enum.
/// The string form is singular lowercase (e.g. "issue", "plan", "doc").
pub fn resolve_target_type(proto_type: LinkTargetType, type_string: &str) -> TargetType {
    if type_string.is_empty() {
        proto_link_target_to_internal(proto_type)
    } else {
        TargetType::new(type_string.to_lowercase())
    }
}

pub fn internal_target_type_to_proto(internal_type: &TargetType) -> i32 {
    match internal_type.as_str() {
        "issue" => LinkTargetType::Issue as i32,
        "doc" => LinkTargetType::Doc as i32,
        _ => LinkTargetType::Unspecified as i32,
    }
}

pub fn link_view_to_proto(view: &LinkView) -> ProtoLink {
    ProtoLink {
        id: view.id.clone(),
        target_id: view.target_id.clone(),
        target_type: internal_target_type_to_proto(&view.target_type),
        link_type: view.link_type.clone(),
        created_at: view.created_at.clone(),
        target_item_type: view.target_type.as_str().to_string(),
        direction: view.direction.as_str().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_uses_string_over_enum() {
        // String field must win over the legacy enum.
        // This is the core of issue #361: the target type prefix must not be ignored.
        let ty = resolve_target_type(LinkTargetType::Unspecified, "plan");
        assert_eq!(ty, TargetType::new("plan"));
    }

    #[test]
    fn resolve_lowercases_the_string_field() {
        let ty = resolve_target_type(LinkTargetType::Unspecified, "Plan");
        assert_eq!(ty, TargetType::new("plan"));
    }

    #[test]
    fn resolve_falls_back_to_enum_when_string_empty() {
        // Empty string: use legacy enum (doc enum gives "doc").
        let ty = resolve_target_type(LinkTargetType::Doc, "");
        assert_eq!(ty, TargetType::new("doc"));
    }

    #[test]
    fn resolve_unspecified_enum_empty_string_defaults_to_issue() {
        let ty = resolve_target_type(LinkTargetType::Unspecified, "");
        assert_eq!(ty, TargetType::issue());
    }

    #[test]
    fn resolve_string_doc_matches_enum_doc() {
        let via_string = resolve_target_type(LinkTargetType::Unspecified, "doc");
        let via_enum = resolve_target_type(LinkTargetType::Doc, "");
        assert_eq!(via_string, via_enum);
    }
}
