use crate::item::entities::issue::{create_issue, list_issues, CreateIssueOptions};
use crate::utils::CENTY_VERSION;

pub async fn create_update_issue(project_path: &str, version: &str) -> (bool, String) {
    let tag = format!("[centy-update:{version}]");
    let path = std::path::Path::new(project_path);

    // Check for duplicate issues
    if let Ok(issues) = list_issues(path, None, None, None, false).await {
        if issues.iter().any(|i| i.title.contains(&tag)) {
            return (false, String::new());
        }
    }

    let title = format!("Update centy-daemon to {version} {tag}");
    let current = CENTY_VERSION;
    let description = format!(
        "A new version of centy-daemon ({version}) is available.\n\n\
         Current version: {current}\n\
         Latest version: {version}\n\n\
         Visit https://github.com/centy-io/centy-daemon/releases for details."
    );

    let options = CreateIssueOptions {
        title,
        description,
        ..Default::default()
    };

    match create_issue(path, options).await {
        Ok(result) => (true, result.id),
        Err(_) => (false, String::new()),
    }
}
