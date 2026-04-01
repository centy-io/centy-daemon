use crate::link::LinkView;

use super::proto::Link as ProtoLink;

pub fn link_view_to_proto(view: &LinkView) -> ProtoLink {
    ProtoLink {
        id: view.id.clone(),
        target_id: view.target_id.clone(),
        link_type: view.link_type.clone(),
        created_at: view.created_at.clone(),
        target_item_type: view.target_type.as_str().to_string(),
        direction: view.direction.as_str().to_string(),
    }
}

#[cfg(test)]
#[path = "convert_link_tests.rs"]
mod convert_link_tests;
