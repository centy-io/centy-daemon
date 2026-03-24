use mdstore::Filters;
use std::collections::HashMap;

/// Extract `customFields` constraints from a JSON-encoded MQL query string.
///
/// Returns a map of field name → required string value.
/// Only exact-match (`{"customFields": {"field": "value"}}`) is supported.
pub(super) fn parse_custom_field_filters(filter_json: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if filter_json.is_empty() {
        return result;
    }
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(filter_json) else {
        return result;
    };
    let Some(obj) = doc.as_object() else {
        return result;
    };
    let Some(cf) = obj.get("customFields").and_then(|v| v.as_object()) else {
        return result;
    };
    for (field, value) in cf {
        if let Some(s) = value.as_str() {
            result.insert(field.clone(), s.to_string());
        }
    }
    result
}

/// Build a `Filters` from a JSON-encoded MQL query string and pagination params.
pub(super) fn build_filters_from_mql(filter_json: &str, limit: u32, offset: u32) -> Filters {
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
            "tags" => {
                filters = apply_tags_condition(filters, condition);
            }
            _ => {}
        }
    }
    filters
}
fn apply_tags_condition(filters: Filters, condition: &serde_json::Value) -> Filters {
    match condition {
        serde_json::Value::String(s) => filters.with_tags_any(vec![s.clone()]),
        serde_json::Value::Object(ops) => {
            if let Some(arr) = ops.get("$in").and_then(|v| v.as_array()) {
                let tags: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect();
                if !tags.is_empty() {
                    return filters.with_tags_any(tags);
                }
            }
            if let Some(arr) = ops.get("$all").and_then(|v| v.as_array()) {
                let tags: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect();
                if !tags.is_empty() {
                    return filters.with_tags_all(tags);
                }
            }
            filters
        }
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::Array(_) => filters,
    }
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
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::Array(_) => filters,
    }
}
fn apply_priority_condition(filters: Filters, condition: &serde_json::Value) -> Filters {
    match condition {
        serde_json::Value::Number(n) => {
            if let Some(p) = n.as_u64() {
                filters.with_priority(u32::try_from(p).unwrap_or(u32::MAX))
            } else {
                filters
            }
        }
        serde_json::Value::Object(ops) => {
            let mut f = filters;
            if let Some(v) = ops.get("$eq").and_then(serde_json::Value::as_u64) {
                f = f.with_priority(u32::try_from(v).unwrap_or(u32::MAX));
            }
            if let Some(v) = ops.get("$lte").and_then(serde_json::Value::as_u64) {
                f = f.with_priority_lte(u32::try_from(v).unwrap_or(u32::MAX));
            }
            if let Some(v) = ops.get("$lt").and_then(serde_json::Value::as_u64) {
                f = f.with_priority_lte(u32::try_from(v.saturating_sub(1)).unwrap_or(u32::MAX));
            }
            if let Some(v) = ops.get("$gte").and_then(serde_json::Value::as_u64) {
                f = f.with_priority_gte(u32::try_from(v).unwrap_or(0));
            }
            if let Some(v) = ops.get("$gt").and_then(serde_json::Value::as_u64) {
                f = f.with_priority_gte(u32::try_from(v.saturating_add(1)).unwrap_or(u32::MAX));
            }
            f
        }
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::String(_)
        | serde_json::Value::Array(_) => filters,
    }
}
