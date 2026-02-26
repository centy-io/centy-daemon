mod convert;
mod defaults;
mod io;
mod registry;
mod types;

pub use convert::{default_archived_config, default_issue_config};
#[allow(unused_imports)]
pub use defaults::{default_doc_config, validate_item_type_config};
#[allow(unused_imports)]
pub use io::{discover_item_types, read_item_type_config, write_item_type_config};
pub use registry::ItemTypeRegistry;
pub use types::{ItemTypeConfig, ItemTypeFeatures};

use super::CentyConfig;
use std::path::Path;

/// Create `config.yaml` for issues, docs, and archived if they don't already exist.
/// Returns the list of relative paths that were created.
pub async fn migrate_to_item_type_configs(
    project_path: &Path,
    config: &CentyConfig,
) -> Result<Vec<String>, mdstore::ConfigError> {
    let mut created = Vec::new();

    let centy_path = crate::utils::get_centy_path(project_path);

    // Issues
    let issues_config_path = centy_path.join("issues").join("config.yaml");
    if !issues_config_path.exists() {
        let issue_config = default_issue_config(config);
        write_item_type_config(project_path, "issues", &issue_config).await?;
        created.push("issues/config.yaml".to_string());
    }

    // Docs
    let docs_config_path = centy_path.join("docs").join("config.yaml");
    if !docs_config_path.exists() {
        let doc_config = default_doc_config();
        write_item_type_config(project_path, "docs", &doc_config).await?;
        created.push("docs/config.yaml".to_string());
    }

    // Archived
    let archived_config_path = centy_path.join("archived").join("config.yaml");
    if !archived_config_path.exists() {
        let archived_config = default_archived_config();
        write_item_type_config(project_path, "archived", &archived_config).await?;
        created.push("archived/config.yaml".to_string());
    }

    Ok(created)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::field_reassign_with_default)]
#[path = "itc_tests_1.rs"]
mod itc_tests_1;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::field_reassign_with_default)]
#[path = "itc_tests_2.rs"]
mod itc_tests_2;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::field_reassign_with_default)]
#[path = "itc_tests_3.rs"]
mod itc_tests_3;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::field_reassign_with_default)]
#[path = "itc_tests_4.rs"]
mod itc_tests_4;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::field_reassign_with_default)]
#[path = "itc_tests_5.rs"]
mod itc_tests_5;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::field_reassign_with_default)]
#[path = "itc_tests_6.rs"]
mod itc_tests_6;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::field_reassign_with_default)]
#[path = "itc_tests_7.rs"]
mod itc_tests_7;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::field_reassign_with_default)]
#[path = "itc_tests_8.rs"]
mod itc_tests_8;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::field_reassign_with_default)]
#[path = "itc_tests_9.rs"]
mod itc_tests_9;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::field_reassign_with_default)]
#[path = "itc_tests_10.rs"]
mod itc_tests_10;
