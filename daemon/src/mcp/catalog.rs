use prost_reflect::{DescriptorPool, MethodDescriptor};
use rmcp::model::{Tool, ToolAnnotations};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct Catalog {
    methods: BTreeMap<String, MethodDescriptor>,
    tools: Vec<Tool>,
}

impl Catalog {
    pub fn from_descriptor(descriptor: &[u8]) -> color_eyre::eyre::Result<Self> {
        let pool = DescriptorPool::decode(descriptor)?;
        let service = pool
            .get_service_by_name("centy.v1.CentyDaemon")
            .ok_or_else(|| color_eyre::eyre::eyre!("CentyDaemon descriptor is missing"))?;
        let methods: BTreeMap<String, MethodDescriptor> = service
            .methods()
            .map(|method| (tool_name(&method), method))
            .collect();
        let tools = methods.values().map(tool_for).collect();
        Ok(Self { methods, tools })
    }

    pub fn method(&self, name: &str) -> Option<&MethodDescriptor> {
        self.methods.get(name)
    }

    pub fn tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }
}

fn tool_for(method: &MethodDescriptor) -> Tool {
    let name = tool_name(method);
    let mut annotations = ToolAnnotations::default();
    annotations.read_only_hint = Some(is_read_only(method.name()));
    let mut tool = Tool::default();
    tool.name = name.into();
    tool.title = Some(method.name().to_owned());
    tool.description = Some(
        format!(
            "{} RPC. Arguments use protobuf JSON mapping for {}.",
            method.name(),
            method.input().full_name()
        )
        .into(),
    );
    tool.input_schema = Arc::new(input_schema());
    tool.annotations = Some(annotations);
    tool
}

fn tool_name(method: &MethodDescriptor) -> String {
    format!("centy_v1_CentyDaemon_{}", method.name())
}

fn is_read_only(name: &str) -> bool {
    name.starts_with("Get")
        || name.starts_with("List")
        || name.starts_with("Is")
        || name.starts_with("Search")
}

fn input_schema() -> Map<String, Value> {
    json!({
        "type": "object",
        "additionalProperties": true,
        "description": "Use protobuf JSON field names and values for this RPC request."
    })
    .as_object()
    .expect("JSON literal is an object")
    .clone()
}
