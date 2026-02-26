use crate::registry::{get_org_config, update_org_config, OrgCustomFieldDef};
use crate::server::proto::{
    CustomFieldDefinition, OrgConfig as ProtoOrgConfig, UpdateOrgConfigRequest,
    UpdateOrgConfigResponse,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};
pub async fn update_org_config_handler(
    req: UpdateOrgConfigRequest,
) -> Result<Response<UpdateOrgConfigResponse>, Status> {
    let mut config = get_org_config(&req.organization_slug)
        .await
        .unwrap_or_default();
    if req.priority_levels > 0 {
        config.priority_levels = req.priority_levels;
    }
    if !req.custom_fields.is_empty() {
        config.custom_fields = req
            .custom_fields
            .into_iter()
            .map(|f| OrgCustomFieldDef {
                name: f.name,
                default_value: if f.default_value.is_empty() {
                    None
                } else {
                    Some(f.default_value)
                },
                description: None,
            })
            .collect();
    }
    match update_org_config(&req.organization_slug, &config).await {
        Ok(()) => Ok(Response::new(UpdateOrgConfigResponse {
            success: true,
            error: String::new(),
            config: Some(ProtoOrgConfig {
                priority_levels: config.priority_levels,
                custom_fields: config
                    .custom_fields
                    .into_iter()
                    .map(|f| CustomFieldDefinition {
                        name: f.name,
                        default_value: f.default_value.unwrap_or_default(),
                        field_type: "string".to_string(),
                        required: false,
                        enum_values: vec![],
                    })
                    .collect(),
            }),
        })),
        Err(e) => Ok(Response::new(UpdateOrgConfigResponse {
            success: false,
            error: to_error_json(&req.organization_slug, &e),
            config: None,
        })),
    }
}
