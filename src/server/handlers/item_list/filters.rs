use mdstore::Filters;
/// Build a `Filters` from a JSON-encoded MQL query string and pagination params.
pub(super) fn build_filters_from_mql(filter_json: &str, limit: u32, offset: u32) -> Filters {
    let mut filters = Filters::new();
    if limit > 0 { filters = filters.with_limit(limit as usize); }
    if offset > 0 { filters = filters.with_offset(offset as usize); }
    if filter_json.is_empty() { return filters; }
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(filter_json) else { return filters; };
    let Some(obj) = doc.as_object() else { return filters; };
    for (field, condition) in obj {
        match field.as_str() {
            "status" => { filters = apply_status_condition(filters, condition); }
            "priority" => { filters = apply_priority_condition(filters, condition); }
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
                let statuses: Vec<String> = arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string)).collect();
                if !statuses.is_empty() { return filters.with_statuses(statuses); }
            }
            filters
        }
        _ => filters,
    }
}
fn apply_priority_condition(filters: Filters, condition: &serde_json::Value) -> Filters {
    match condition {
        serde_json::Value::Number(n) => {
            if let Some(p) = n.as_u64() { filters.with_priority(p as u32) } else { filters }
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
