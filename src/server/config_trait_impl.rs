use tonic::{Request, Response, Status};

use super::handlers;
use super::proto::config_service_server::ConfigService;
use super::proto::*;
use super::ConfigServiceImpl;

#[tonic::async_trait]
impl ConfigService for ConfigServiceImpl {
    async fn get_manifest(
        &self,
        request: Request<GetManifestRequest>,
    ) -> Result<Response<GetManifestResponse>, Status> {
        handlers::manifest::get_manifest(request.into_inner()).await
    }

    async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<GetConfigResponse>, Status> {
        handlers::config::get_config(request.into_inner()).await
    }

    async fn update_config(
        &self,
        request: Request<UpdateConfigRequest>,
    ) -> Result<Response<UpdateConfigResponse>, Status> {
        handlers::config_update::update_config(request.into_inner()).await
    }
}
