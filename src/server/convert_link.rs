use crate::link::TargetType;

use super::proto::{Link as ProtoLink, LinkTargetType};

pub fn proto_link_target_to_internal(proto_type: LinkTargetType) -> TargetType {
    match proto_type {
        LinkTargetType::Issue | LinkTargetType::Unspecified => TargetType::issue(),
        LinkTargetType::Doc => TargetType::new("doc"),
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
        link_type: link.link_type.clone(),
        created_at: link.created_at.clone(),
    }
}
