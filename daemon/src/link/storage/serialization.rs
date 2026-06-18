use super::super::types::{LinkRecord, TargetType};
use serde_json::json;
use std::collections::HashMap;

pub fn item_to_link_record(item: mdstore::Item) -> Option<LinkRecord> {
    let cf = &item.frontmatter.custom_fields;
    let source_id = cf.get("sourceId")?.as_str()?.to_string();
    let source_type = TargetType::new(cf.get("sourceType")?.as_str()?);
    let target_id = cf.get("targetId")?.as_str()?.to_string();
    let target_type = TargetType::new(cf.get("targetType")?.as_str()?);
    let link_type = cf.get("linkType")?.as_str()?.to_string();
    Some(LinkRecord {
        id: item.id,
        source_id,
        source_type,
        target_id,
        target_type,
        link_type,
        created_at: item.frontmatter.created_at,
        updated_at: item.frontmatter.updated_at,
    })
}

pub fn create_link_fields(
    source_id: &str,
    source_type: &TargetType,
    target_id: &str,
    target_type: &TargetType,
    link_type: &str,
) -> HashMap<String, serde_json::Value> {
    let mut fields = HashMap::new();
    fields.insert("sourceId".to_string(), json!(source_id));
    fields.insert("sourceType".to_string(), json!(source_type.as_str()));
    fields.insert("targetId".to_string(), json!(target_id));
    fields.insert("targetType".to_string(), json!(target_type.as_str()));
    fields.insert("linkType".to_string(), json!(link_type));
    fields
}

pub fn update_link_fields(link_type: &str) -> HashMap<String, serde_json::Value> {
    let mut fields = HashMap::new();
    fields.insert("linkType".to_string(), json!(link_type));
    fields
}
