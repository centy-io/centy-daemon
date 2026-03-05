use handlebars::Handlebars;
use std::path::Path;
use thiserror::Error;
use tokio::fs;

use super::types::{IssueTemplateContext, TemplateType};
use crate::utils::get_centy_path;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Template error: {0}")]
    TemplateError(#[from] handlebars::TemplateError),
    #[error("Render error: {0}")]
    RenderError(#[from] handlebars::RenderError),
    #[error("Template '{0}' not found")]
    TemplateNotFound(String),
}

pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlebars: Handlebars::new(),
        }
    }

    /// Get the templates directory path
    #[must_use]
    pub fn get_templates_path(project_path: &Path) -> std::path::PathBuf {
        get_centy_path(project_path).join("templates")
    }

    /// Get the path for a specific template type's folder
    #[must_use]
    pub fn get_template_type_path(
        project_path: &Path,
        template_type: TemplateType,
    ) -> std::path::PathBuf {
        Self::get_templates_path(project_path).join(template_type.folder_name())
    }

    /// Load a template from disk by name. Looks for "{template_name}.md" in the appropriate folder.
    pub async fn load_template(
        &self,
        project_path: &Path,
        template_type: TemplateType,
        template_name: &str,
    ) -> Result<String, TemplateError> {
        let template_folder = Self::get_template_type_path(project_path, template_type);
        let file_name = format!("{template_name}.md");
        let template_path = template_folder.join(&file_name);
        if template_path.exists() {
            Ok(fs::read_to_string(&template_path).await?)
        } else {
            Err(TemplateError::TemplateNotFound(file_name))
        }
    }

    /// Render an issue using a template
    pub async fn render_issue(
        &self,
        project_path: &Path,
        template_name: &str,
        context: &IssueTemplateContext,
    ) -> Result<String, TemplateError> {
        let template_content = self
            .load_template(project_path, TemplateType::Issue, template_name)
            .await?;
        self.handlebars
            .render_template(&template_content, context)
            .map_err(TemplateError::from)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
