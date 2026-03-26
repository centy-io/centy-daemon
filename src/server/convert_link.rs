use crate::link::TargetType;

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

pub fn internal_link_to_proto(link: &crate::link::Link) -> ProtoLink {
    ProtoLink {
        target_id: link.target_id.clone(),
        target_type: internal_target_type_to_proto(&link.target_type),
        link_type: link.kind.clone(),
        created_at: link.created_at.clone(),
        target_item_type: link.target_type.as_str().to_string(),
    }
}
