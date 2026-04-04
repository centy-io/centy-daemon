use std::path::Path;
use tokio::fs;

use super::mcp_json::{initial_mcp_json, inject_centy_entry, MCP_JSON_FILENAME};

/// Ensures `.mcp.json` in the project root contains the centy MCP server entry.
///
/// Behavior:
/// - File does not exist → create it with the centy entry
/// - File exists, `mcpServers.centy` absent → inject only the `centy` key
/// - File exists, `mcpServers.centy` already present → no-op
/// - File exists but invalid JSON → return an error, do not modify
pub async fn ensure_mcp_json(project_path: &Path) -> Result<(), String> {
    let mcp_path = project_path.join(MCP_JSON_FILENAME);

    if !mcp_path.exists() {
        fs::write(&mcp_path, initial_mcp_json())
            .await
            .map_err(|e| format!("Failed to write .mcp.json: {e}"))?;
        return Ok(());
    }

    let raw = fs::read_to_string(&mcp_path)
        .await
        .map_err(|e| format!("Failed to read .mcp.json: {e}"))?;

    if let Some(updated) = inject_centy_entry(&raw)? {
        fs::write(&mcp_path, updated)
            .await
            .map_err(|e| format!("Failed to write .mcp.json: {e}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tempfile::tempdir;

    #[tokio::test]
    async fn creates_mcp_json_when_absent() {
        let dir = tempdir().unwrap();
        ensure_mcp_json(dir.path()).await.unwrap();
        let path = dir.path().join(".mcp.json");
        assert!(path.exists());
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        let doc: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(doc["mcpServers"]["centy"]["command"], "npx");
        assert_eq!(doc["mcpServers"]["centy"]["args"][0], "-y");
        assert_eq!(doc["mcpServers"]["centy"]["args"][1], "centy-mcp");
    }

    #[tokio::test]
    async fn injects_centy_into_existing_file_without_centy() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".mcp.json");
        let existing = r#"{"mcpServers":{"other":{"command":"other-cmd","args":[]}}}"#;
        tokio::fs::write(&path, existing).await.unwrap();

        ensure_mcp_json(dir.path()).await.unwrap();

        let content = tokio::fs::read_to_string(&path).await.unwrap();
        let doc: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(doc["mcpServers"]["centy"]["command"], "npx");
        // Other keys preserved
        assert_eq!(doc["mcpServers"]["other"]["command"], "other-cmd");
    }

    #[tokio::test]
    async fn no_op_when_centy_already_present() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".mcp.json");
        let existing = r#"{"mcpServers":{"centy":{"command":"custom","args":["--custom"]}}}"#;
        tokio::fs::write(&path, existing).await.unwrap();

        ensure_mcp_json(dir.path()).await.unwrap();

        let content = tokio::fs::read_to_string(&path).await.unwrap();
        let doc: Value = serde_json::from_str(&content).unwrap();
        // Existing custom config must not be overwritten
        assert_eq!(doc["mcpServers"]["centy"]["command"], "custom");
        assert_eq!(doc["mcpServers"]["centy"]["args"][0], "--custom");
    }

    #[tokio::test]
    async fn errors_on_invalid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".mcp.json");
        tokio::fs::write(&path, b"not valid json").await.unwrap();

        let result = ensure_mcp_json(dir.path()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("invalid JSON"), "error was: {err}");
        // File must not be modified
        let still_invalid = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(still_invalid, "not valid json");
    }

    #[tokio::test]
    async fn idempotent_when_called_twice() {
        let dir = tempdir().unwrap();
        ensure_mcp_json(dir.path()).await.unwrap();
        let first = tokio::fs::read_to_string(dir.path().join(".mcp.json"))
            .await
            .unwrap();
        ensure_mcp_json(dir.path()).await.unwrap();
        let second = tokio::fs::read_to_string(dir.path().join(".mcp.json"))
            .await
            .unwrap();
        assert_eq!(first, second);
    }
}
