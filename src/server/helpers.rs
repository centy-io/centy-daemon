use crate::common::org_sync::OrgSyncResult;
use crate::item::entities::doc::OrgDocSyncResult as DomainOrgDocSyncResult;

use super::proto::OrgDocSyncResult;

/// Convert an empty string to `None`, non-empty to `Some`.
pub fn nonempty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Convert a protobuf `int32` to `Option<u32>`: 0 means "not set".
pub fn nonzero_u32(v: i32) -> Option<u32> {
    if v == 0 {
        None
    } else {
        Some(v as u32)
    }
}

/// Convert internal `OrgSyncResult` vec to the proto representation.
pub fn sync_results_to_proto(results: Vec<OrgSyncResult>) -> Vec<OrgDocSyncResult> {
    results
        .into_iter()
        .map(|r| OrgDocSyncResult {
            project_path: r.project_path,
            success: r.success,
            error: r.error.unwrap_or_default(),
        })
        .collect()
}

/// Convert internal doc `OrgDocSyncResult` vec to the proto representation.
pub fn doc_sync_results_to_proto(results: Vec<DomainOrgDocSyncResult>) -> Vec<OrgDocSyncResult> {
    results
        .into_iter()
        .map(|r| OrgDocSyncResult {
            project_path: r.project_path,
            success: r.success,
            error: r.error.unwrap_or_default(),
        })
        .collect()
}
