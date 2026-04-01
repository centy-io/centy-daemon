use super::*;
use crate::link::{LinkDirection, LinkView, TargetType};

#[test]
fn link_view_to_proto_issue_type() {
    let view = LinkView {
        id: "link-id".to_string(),
        target_id: "target-uuid".to_string(),
        target_type: TargetType::issue(),
        link_type: "blocks".to_string(),
        direction: LinkDirection::Source,
        created_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let proto = link_view_to_proto(&view);
    assert_eq!(proto.id, "link-id");
    assert_eq!(proto.target_id, "target-uuid");
    assert_eq!(proto.link_type, "blocks");
    assert_eq!(proto.target_item_type, "issue");
    assert_eq!(proto.created_at, "2024-01-01T00:00:00Z");
    assert_eq!(proto.direction, "source");
}

#[test]
fn link_view_to_proto_doc_type() {
    let view = LinkView {
        id: "link-id".to_string(),
        target_id: "doc-uuid".to_string(),
        target_type: TargetType::new("doc"),
        link_type: "references".to_string(),
        direction: LinkDirection::Target,
        created_at: "2024-06-01T00:00:00Z".to_string(),
    };
    let proto = link_view_to_proto(&view);
    assert_eq!(proto.target_id, "doc-uuid");
    assert_eq!(proto.target_item_type, "doc");
    assert_eq!(proto.direction, "target");
}

#[test]
fn link_view_to_proto_custom_type() {
    let view = LinkView {
        id: "link-id".to_string(),
        target_id: "epic-uuid".to_string(),
        target_type: TargetType::new("epic"),
        link_type: "relates-to".to_string(),
        direction: LinkDirection::Source,
        created_at: "2024-06-01T00:00:00Z".to_string(),
    };
    let proto = link_view_to_proto(&view);
    assert_eq!(proto.target_item_type, "epic");
}
