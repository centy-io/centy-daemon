use std::path::Path;

use crate::hooks::{HookItemType, HookOperation};
use crate::item::entities::doc::{create_doc, CreateDocOptions};
use crate::registry::track_project_async;
use crate::server::convert_infra::manifest_to_proto;
use crate::server::helpers::{doc_sync_results_to_proto, nonempty};
use crate::server::hooks_helper::{maybe_run_post_hooks, maybe_run_pre_hooks};
use crate::server::proto::{CreateDocRequest, CreateDocResponse};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn create_doc_handler(
    req: CreateDocRequest,
) -> Result<Response<CreateDocResponse>, Status> {
    track_project_async(req.project_path.clone());
    let project_path = Path::new(&req.project_path);
    let hook_project_path = req.project_path.clone();
    let hook_request_data = serde_json::json!({
        "title": &req.title,
        "content": &req.content,
        "slug": &req.slug,
    });
    if let Err(e) = maybe_run_pre_hooks(
        project_path,
        HookItemType::Doc,
        HookOperation::Create,
        &hook_project_path,
        None,
        Some(hook_request_data.clone()),
    )
    .await
    {
        return Ok(Response::new(CreateDocResponse {
            success: false,
            error: to_error_json(&req.project_path, &e),
            ..Default::default()
        }));
    }

    let options = CreateDocOptions {
        title: req.title,
        content: req.content,
        slug: nonempty(req.slug),
        template: nonempty(req.template),
        is_org_doc: req.is_org_doc,
    };

    match create_doc(project_path, options).await {
        Ok(result) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Doc,
                HookOperation::Create,
                &hook_project_path,
                Some(&result.slug),
                Some(hook_request_data),
                true,
            )
            .await;
            let sync_results = doc_sync_results_to_proto(result.sync_results);
            Ok(Response::new(CreateDocResponse {
                success: true,
                error: String::new(),
                slug: result.slug,
                created_file: result.created_file,
                manifest: Some(manifest_to_proto(&result.manifest)),
                sync_results,
            }))
        }
        Err(e) => {
            maybe_run_post_hooks(
                project_path,
                HookItemType::Doc,
                HookOperation::Create,
                &hook_project_path,
                None,
                Some(hook_request_data),
                false,
            )
            .await;
            Ok(Response::new(CreateDocResponse {
                success: false,
                error: to_error_json(&req.project_path, &e),
                slug: String::new(),
                created_file: String::new(),
                manifest: None,
                sync_results: Vec::new(),
            }))
        }
    }
}
