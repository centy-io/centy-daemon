use serde_json::{json, Value};

pub const MCP_JSON_FILENAME: &str = ".mcp.json";

pub fn centy_mcp_entry() -> Value {
    json!({
        "command": "npx",
        "args": ["-y", "centy-mcp"]
    })
}

/// Build the initial `.mcp.json` content with only the centy entry.
pub fn initial_mcp_json() -> String {
    let content = json!({
        "mcpServers": {
            "centy": centy_mcp_entry()
        }
    });
    let mut formatted = serde_json::to_string_pretty(&content)
        .unwrap_or_default();
    formatted.push('\n');
    formatted
}

/// Inject the centy MCP entry into the parsed JSON if absent.
///
/// - Returns `Ok(None)` if `mcpServers.centy` is already present (no-op).
/// - Returns `Ok(Some(updated))` with the formatted JSON string when the entry was injected.
/// - Returns `Err` if `raw` is invalid JSON or the root is not a JSON object.
pub fn inject_centy_entry(raw: &str) -> Result<Option<String>, String> {
    let mut doc: Value =
        serde_json::from_str(raw).map_err(|e| format!(".mcp.json contains invalid JSON: {e}"))?;

    if doc.get("mcpServers").and_then(|s| s.get("centy")).is_some() {
        return Ok(None);
    }

    let root = doc
        .as_object_mut()
        .ok_or_else(|| ".mcp.json root is not a JSON object".to_string())?;

    match root.get_mut("mcpServers") {
        Some(servers) => {
            if let Some(map) = servers.as_object_mut() {
                map.insert("centy".to_string(), centy_mcp_entry());
            } else {
                root.insert(
                    "mcpServers".to_string(),
                    json!({ "centy": centy_mcp_entry() }),
                );
            }
        }
        None => {
            root.insert(
                "mcpServers".to_string(),
                json!({ "centy": centy_mcp_entry() }),
            );
        }
    }

    let mut formatted = serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("Failed to serialize updated .mcp.json: {e}"))?;
    formatted.push('\n');
    Ok(Some(formatted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_mcp_json_has_centy_entry() {
        let content = initial_mcp_json();
        let doc: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(doc["mcpServers"]["centy"]["command"], "npx");
        assert_eq!(doc["mcpServers"]["centy"]["args"][0], "-y");
        assert_eq!(doc["mcpServers"]["centy"]["args"][1], "centy-mcp");
    }

    #[test]
    fn inject_adds_centy_to_existing_servers() {
        let raw = r#"{"mcpServers":{"other":{"command":"other-cmd","args":[]}}}"#;
        let updated = inject_centy_entry(raw).unwrap().expect("should inject");
        let doc: Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(doc["mcpServers"]["centy"]["command"], "npx");
        assert_eq!(doc["mcpServers"]["other"]["command"], "other-cmd");
    }

    #[test]
    fn inject_returns_none_when_centy_present() {
        let raw = r#"{"mcpServers":{"centy":{"command":"custom","args":["--custom"]}}}"#;
        let result = inject_centy_entry(raw).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn inject_errors_on_invalid_json() {
        let result = inject_centy_entry("not valid json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid JSON"));
    }

    #[test]
    fn inject_adds_mcp_servers_key_when_absent() {
        let raw = r#"{"otherKey": 42}"#;
        let updated = inject_centy_entry(raw).unwrap().expect("should inject");
        let doc: Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(doc["mcpServers"]["centy"]["command"], "npx");
        assert_eq!(doc["otherKey"], 42i32);
    }
}
