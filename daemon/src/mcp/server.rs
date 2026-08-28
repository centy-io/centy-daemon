use super::{catalog::Catalog, grpc};
use color_eyre::eyre::Result;
use prost_reflect::DynamicMessage;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, Implementation, ListToolsResult,
    ServerCapabilities, ServerInfo,
};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::ErrorData;
use rmcp::ServerHandler;
use serde_json::Value;
use std::future::Future;
use std::sync::Arc;

pub struct McpServer {
    catalog: Arc<Catalog>,
    grpc_endpoint: String,
}

impl McpServer {
    pub fn new(grpc_endpoint: String) -> Result<Self> {
        Ok(Self {
            catalog: Arc::new(Catalog::from_descriptor(crate::app::FILE_DESCRIPTOR_SET)?),
            grpc_endpoint,
        })
    }

    pub fn into_service(self) -> StreamableHttpService<Self, LocalSessionManager> {
        StreamableHttpService::new(
            move || Ok(self.clone()),
            Arc::default(),
            StreamableHttpServerConfig::default(),
        )
    }
}

impl Clone for McpServer {
    fn clone(&self) -> Self {
        Self {
            catalog: Arc::clone(&self.catalog),
            grpc_endpoint: self.grpc_endpoint.clone(),
        }
    }
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "centy-daemon",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Centy's built-in local MCP server. Every CentyDaemon RPC is a tool.",
            )
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = std::result::Result<ListToolsResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult {
            tools: self.catalog.tools(),
            ..Default::default()
        }))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> std::result::Result<CallToolResponse, ErrorData> {
        let result = match self.catalog.method(&request.name).cloned() {
            Some(method) => {
                call(
                    &self.grpc_endpoint,
                    method,
                    request.arguments.unwrap_or_default(),
                )
                .await
            }
            None => CallToolResult::structured_error(
                serde_json::json!({"error": "unknown Centy RPC tool"}),
            ),
        };
        Ok(result.into())
    }
}

async fn call(
    grpc_endpoint: &str,
    method: prost_reflect::MethodDescriptor,
    arguments: serde_json::Map<String, Value>,
) -> CallToolResult {
    let json = Value::Object(arguments).to_string();
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    let input = match DynamicMessage::deserialize(method.input(), &mut deserializer) {
        Ok(value) => value,
        Err(error) => {
            return CallToolResult::structured_error(
                serde_json::json!({"error": error.to_string()}),
            )
        }
    };
    let rpc_path = format!("/{}/{}", method.parent_service().full_name(), method.name());
    let path: http::uri::PathAndQuery = match rpc_path.parse() {
        Ok(value) => value,
        Err(error) => {
            return CallToolResult::structured_error(
                serde_json::json!({"error": error.to_string()}),
            );
        }
    };
    let response = match grpc::unary(grpc_endpoint, path, input, method.output()).await {
        Ok(response) => response,
        Err(error) => {
            return CallToolResult::structured_error(
                serde_json::json!({"error": error.to_string()}),
            )
        }
    };
    serde_json::to_value(response).map_or_else(
        |error| CallToolResult::structured_error(serde_json::json!({"error": error.to_string()})),
        CallToolResult::structured,
    )
}
