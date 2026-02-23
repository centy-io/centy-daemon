use std::path::Path;

use crate::item::generic::storage::generic_list;
use crate::registry::track_project_async;
use crate::server::convert_entity::generic_item_to_proto;
use crate::server::proto::{ListItemsRequest, ListItemsResponse};
use crate::server::structured_error::to_error_json;
use mdstore::Filters;
use tonic::{Response, Status};

use super::item_type_resolve::resolve_item_type_config;

pub async fn list_items(req: ListItemsRequest) -> Result<Response<ListItemsResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let (item_type, _config) = match resolve_item_type_config(project_path, &req.item_type).await {
        Ok(pair) => pair,
        Err(e) => {
            return Ok(Response::new(ListItemsResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                items: vec![],
                total_count: 0,
            }));
        }
    };

    let filters = build_filters_from_mql(&req.filter, req.limit, req.offset);

    match generic_list(project_path, &item_type, filters).await {
        Ok(items) => {
            let total_count = items.len() as i32;
            Ok(Response::new(ListItemsResponse {
                success: true,
                error: String::new(),
                items: items
                    .iter()
                    .map(|item| generic_item_to_proto(item, &item_type))
                    .collect(),
                total_count,
            }))
        }
        Err(e) => Ok(Response::new(ListItemsResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            items: vec![],
            total_count: 0,
        })),
    }
}

/// Build a `Filters` from a JSON-encoded MQL query string and pagination params.
///
/// Supported MQL fields and operators:
/// - `status`: `"value"` or `{"$in": ["v1", "v2"]}`
/// - `priority`: `N` or `{"$eq": N, "$lte": N, "$gte": N, "$lt": N, "$gt": N}`
/// - `deletedAt`: `{"$exists": true}` to include soft-deleted items
fn build_filters_from_mql(filter_json: &str, limit: u32, offset: u32) -> Filters {
    let mut filters = Filters::new();

    if limit > 0 {
        filters = filters.with_limit(limit as usize);
    }
    if offset > 0 {
        filters = filters.with_offset(offset as usize);
    }

    if filter_json.is_empty() {
        return filters;
    }

    let Ok(doc) = serde_json::from_str::<serde_json::Value>(filter_json) else {
        return filters;
    };

    let Some(obj) = doc.as_object() else {
        return filters;
    };

    for (field, condition) in obj {
        match field.as_str() {
            "status" => {
                filters = apply_status_condition(filters, condition);
            }
            "priority" => {
                filters = apply_priority_condition(filters, condition);
            }
            "deletedAt" => {
                if let Some(ops) = condition.as_object() {
                    if ops.get("$exists").and_then(serde_json::Value::as_bool) == Some(true) {
                        filters = filters.include_deleted();
                    }
                }
            }
            _ => {}
        }
    }

    filters
}

fn apply_status_condition(filters: Filters, condition: &serde_json::Value) -> Filters {
    match condition {
        serde_json::Value::String(s) => filters.with_statuses(vec![s.clone()]),
        serde_json::Value::Object(ops) => {
            if let Some(v) = ops.get("$eq").and_then(serde_json::Value::as_str) {
                return filters.with_statuses(vec![v.to_string()]);
            }
            if let Some(arr) = ops.get("$in").and_then(|v| v.as_array()) {
                let statuses: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect();
                if !statuses.is_empty() {
                    return filters.with_statuses(statuses);
                }
            }
            filters
        }
        _ => filters,
    }
}

fn apply_priority_condition(filters: Filters, condition: &serde_json::Value) -> Filters {
    match condition {
        serde_json::Value::Number(n) => {
            if let Some(p) = n.as_u64() {
                filters.with_priority(p as u32)
            } else {
                filters
            }
        }
        serde_json::Value::Object(ops) => {
            let mut f = filters;
            if let Some(v) = ops.get("$eq").and_then(serde_json::Value::as_u64) {
                f = f.with_priority(v as u32);
            }
            if let Some(v) = ops.get("$lte").and_then(serde_json::Value::as_u64) {
                f = f.with_priority_lte(v as u32);
            }
            if let Some(v) = ops.get("$lt").and_then(serde_json::Value::as_u64) {
                f = f.with_priority_lte(v.saturating_sub(1) as u32);
            }
            if let Some(v) = ops.get("$gte").and_then(serde_json::Value::as_u64) {
                f = f.with_priority_gte(v as u32);
            }
            if let Some(v) = ops.get("$gt").and_then(serde_json::Value::as_u64) {
                f = f.with_priority_gte(v.saturating_add(1) as u32);
            }
            f
        }
        _ => filters,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_filter_returns_defaults() {
        let f = build_filters_from_mql("", 0, 0);
        assert!(f.statuses.is_none());
        assert!(f.priority.is_none());
        assert!(!f.include_deleted);
        assert!(f.limit.is_none());
        assert!(f.offset.is_none());
    }

    #[test]
    fn test_pagination_applied() {
        let f = build_filters_from_mql("", 10, 5);
        assert_eq!(f.limit, Some(10));
        assert_eq!(f.offset, Some(5));
    }

    #[test]
    fn test_status_exact_match() {
        let f = build_filters_from_mql(r#"{"status":"open"}"#, 0, 0);
        assert_eq!(f.statuses, Some(vec!["open".to_string()]));
    }

    #[test]
    fn test_status_in_operator() {
        let f = build_filters_from_mql(r#"{"status":{"$in":["open","in-progress"]}}"#, 0, 0);
        assert_eq!(
            f.statuses,
            Some(vec!["open".to_string(), "in-progress".to_string()])
        );
    }

    #[test]
    fn test_priority_exact() {
        let f = build_filters_from_mql(r#"{"priority":1}"#, 0, 0);
        assert_eq!(f.priority, Some(1));
    }

    #[test]
    fn test_priority_lte() {
        let f = build_filters_from_mql(r#"{"priority":{"$lte":2}}"#, 0, 0);
        assert_eq!(f.priority_lte, Some(2));
    }

    #[test]
    fn test_priority_gte() {
        let f = build_filters_from_mql(r#"{"priority":{"$gte":1}}"#, 0, 0);
        assert_eq!(f.priority_gte, Some(1));
    }

    #[test]
    fn test_deleted_at_exists() {
        let f = build_filters_from_mql(r#"{"deletedAt":{"$exists":true}}"#, 0, 0);
        assert!(f.include_deleted);
    }

    #[test]
    fn test_invalid_json_returns_defaults() {
        let f = build_filters_from_mql("not-json", 0, 0);
        assert!(f.statuses.is_none());
        assert!(f.priority.is_none());
    }

    #[test]
    fn test_combined_filter() {
        let f = build_filters_from_mql(
            r#"{"status":{"$in":["open","in-progress"]},"priority":{"$lte":2}}"#,
            20,
            0,
        );
        assert_eq!(
            f.statuses,
            Some(vec!["open".to_string(), "in-progress".to_string()])
        );
        assert_eq!(f.priority_lte, Some(2));
        assert_eq!(f.limit, Some(20));
    }
}
