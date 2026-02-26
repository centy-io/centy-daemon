use super::super::metadata::IssueFrontmatter;
use super::super::priority::priority_label;
use super::types::{CreateIssueOptions, IssueError};
use crate::template::{IssueTemplateContext, TemplateEngine};
use std::path::Path;

/// Generate the display title and description, applying a template if specified.
pub async fn render_title_and_description(
    project_path: &Path,
    options: &CreateIssueOptions,
    priority: u32,
    priority_levels: u32,
    status: &str,
    frontmatter: &IssueFrontmatter,
) -> Result<(String, String), IssueError> {
    if let Some(ref template_name) = options.template {
        let template_engine = TemplateEngine::new();
        let context = IssueTemplateContext {
            title: options.title.clone(),
            description: options.description.clone(),
            priority,
            priority_label: priority_label(priority, priority_levels),
            status: status.to_string(),
            created_at: frontmatter.created_at.clone(),
            custom_fields: options.custom_fields.clone(),
        };
        let templated = template_engine
            .render_issue(project_path, template_name, &context)
            .await?;

        let (extracted_title, desc) = parse_templated_content(&templated);
        let title = if extracted_title.is_empty() {
            options.title.clone()
        } else {
            extracted_title
        };
        Ok((title, desc))
    } else {
        Ok((options.title.clone(), options.description.clone()))
    }
}

/// Parse templated content to extract just the description (body without title)
pub fn parse_templated_content(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return (String::new(), String::new());
    }

    let mut title_idx = 0;
    for (idx, line) in lines.iter().enumerate() {
        if line.starts_with('#') {
            title_idx = idx;
            break;
        }
    }

    let title = lines
        .get(title_idx)
        .map(|line| line.strip_prefix('#').map_or(*line, str::trim))
        .unwrap_or("")
        .to_string();

    let description = lines
        .get(title_idx.saturating_add(1)..)
        .unwrap_or(&[])
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string();

    (title, description)
}
