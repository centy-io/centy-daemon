use crate::item::entities::issue::AssetInfo;
use crate::manifest::ManagedFileType as InternalFileType;
use crate::registry::{OrgInferenceResult, OrganizationInfo, ProjectInfo};
use crate::utils::format_display_path;

use super::proto::{
    Asset, FileInfo, FileType, Manifest, OrgInferenceResult as ProtoOrgInferenceResult,
    Organization as ProtoOrganization,
};
use crate::manifest::CentyManifest as InternalManifest;

pub fn project_info_to_proto(info: &ProjectInfo) -> super::proto::ProjectInfo {
    super::proto::ProjectInfo {
        path: info.path.clone(),
        first_accessed: info.first_accessed.clone(),
        last_accessed: info.last_accessed.clone(),
        issue_count: info.issue_count,
        doc_count: info.doc_count,
        initialized: info.initialized,
        name: info.name.clone().unwrap_or_default(),
        is_favorite: info.is_favorite,
        is_archived: info.is_archived,
        display_path: format_display_path(&info.path),
        organization_slug: info.organization_slug.clone().unwrap_or_default(),
        organization_name: info.organization_name.clone().unwrap_or_default(),
        user_title: info.user_title.clone().unwrap_or_default(),
        project_title: info.project_title.clone().unwrap_or_default(),
    }
}

pub fn org_info_to_proto(info: &OrganizationInfo) -> ProtoOrganization {
    ProtoOrganization {
        slug: info.slug.clone(),
        name: info.name.clone(),
        description: info.description.clone().unwrap_or_default(),
        created_at: info.created_at.clone(),
        updated_at: info.updated_at.clone(),
        project_count: info.project_count,
    }
}

pub fn org_inference_to_proto(result: &OrgInferenceResult) -> ProtoOrgInferenceResult {
    ProtoOrgInferenceResult {
        inferred_org_slug: result.inferred_org_slug.clone().unwrap_or_default(),
        inferred_org_name: result.inferred_org_name.clone().unwrap_or_default(),
        org_created: result.org_created,
        existing_org_slug: result.existing_org_slug.clone().unwrap_or_default(),
        has_mismatch: result.has_mismatch,
        message: result.message.clone().unwrap_or_default(),
    }
}

pub fn asset_info_to_proto(asset: &AssetInfo) -> Asset {
    Asset {
        filename: asset.filename.clone(),
        hash: asset.hash.clone(),
        size: asset.size,
        mime_type: asset.mime_type.clone(),
        is_shared: asset.is_shared,
        created_at: asset.created_at.clone(),
    }
}

pub fn manifest_to_proto(manifest: &InternalManifest) -> Manifest {
    Manifest {
        schema_version: manifest.schema_version as i32,
        centy_version: manifest.centy_version.clone(),
        created_at: manifest.created_at.clone(),
        updated_at: manifest.updated_at.clone(),
    }
}

pub fn file_info_to_proto(info: crate::reconciliation::FileInfo) -> FileInfo {
    FileInfo {
        path: info.path,
        file_type: match info.file_type {
            InternalFileType::File => FileType::File as i32,
            InternalFileType::Directory => FileType::Directory as i32,
        },
        hash: info.hash,
        content_preview: info.content_preview.unwrap_or_default(),
    }
}
