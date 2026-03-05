use crate::registry::get_org_config;
use crate::server::proto::{
    CustomFieldDefinition, GetOrgConfigRequest, OrgConfig as ProtoOrgConfig,
};
use crate::server::structured_error::to_error_json;
use tonic::{Response, Status};

pub async fn get_org_config_handler(
    req: GetOrgConfigRequest,
) -> Result<Response<ProtoOrgConfig>, Status> {
    match get_org_config(&req.organization_slug).await {
        Ok(config) => Ok(Response::new(ProtoOrgConfig {
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
        })),
        Err(e) => Err(Status::internal(to_error_json(&req.organization_slug, &e))),
    }
}
