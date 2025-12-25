use crate::manifest::{
    read_manifest, write_manifest, update_manifest_timestamp, CentyManifest,
};
use crate::registry::ProjectInfo;
use crate::template::{DocTemplateContext, TemplateEngine, TemplateError};
use crate::utils::{format_markdown, get_centy_path, now_iso};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum DocError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Centy not initialized. Run 'centy init' first.")]
    NotInitialized,

    #[error("Doc '{0}' not found")]
    DocNotFound(String),

    #[error("Title is required")]
    TitleRequired,

    #[error("Doc with slug '{0}' already exists")]
    SlugAlreadyExists(String),

    #[error("Invalid slug: {0}")]
    InvalidSlug(String),

    #[error("Template error: {0}")]
    TemplateError(#[from] TemplateError),

    #[error("Target project not initialized")]
    TargetNotInitialized,

    #[error("Cannot move doc to same project")]
    SameProjectMove,
}

/// Full doc data
#[derive(Debug, Clone)]
pub struct Doc {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub metadata: DocMetadata,
}

/// Doc metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocMetadata {
    pub created_at: String,
    pub updated_at: String,
}

impl DocMetadata {
    #[must_use] 
    pub fn new() -> Self {
        let now = now_iso();
        Self {
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl Default for DocMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for creating a doc
#[derive(Debug, Clone, Default)]
pub struct CreateDocOptions {
    pub title: String,
    pub content: String,
    pub slug: Option<String>,
    /// Optional template name (without .md extension)
    pub template: Option<String>,
}

/// Result of doc creation
#[derive(Debug, Clone)]
pub struct CreateDocResult {
    pub slug: String,
    pub created_file: String,
    pub manifest: CentyManifest,
}

/// Options for updating a doc
#[derive(Debug, Clone, Default)]
pub struct UpdateDocOptions {
    pub title: Option<String>,
    pub content: Option<String>,
    pub new_slug: Option<String>,
}

/// Result of doc update
#[derive(Debug, Clone)]
pub struct UpdateDocResult {
    pub doc: Doc,
    pub manifest: CentyManifest,
}

/// Result of doc deletion
#[derive(Debug, Clone)]
pub struct DeleteDocResult {
    pub manifest: CentyManifest,
}

/// A doc with its source project information
#[derive(Debug, Clone)]
pub struct DocWithProject {
    pub doc: Doc,
    pub project_path: String,
    pub project_name: String,
}

/// Result of searching for docs by slug across projects
#[derive(Debug, Clone)]
pub struct GetDocsBySlugResult {
    pub docs: Vec<DocWithProject>,
    pub errors: Vec<String>,
}

/// Options for moving a doc to another project
#[derive(Debug, Clone)]
pub struct MoveDocOptions {
    pub source_project_path: PathBuf,
    pub target_project_path: PathBuf,
    pub slug: String,
    pub new_slug: Option<String>,
}

/// Result of moving a doc
#[derive(Debug, Clone)]
pub struct MoveDocResult {
    pub doc: Doc,
    pub old_slug: String,
    pub source_manifest: CentyManifest,
    pub target_manifest: CentyManifest,
}

/// Options for duplicating a doc
#[derive(Debug, Clone)]
pub struct DuplicateDocOptions {
    pub source_project_path: PathBuf,
    pub target_project_path: PathBuf,
    pub slug: String,
    pub new_slug: Option<String>,
    pub new_title: Option<String>,
}

/// Result of duplicating a doc
#[derive(Debug, Clone)]
pub struct DuplicateDocResult {
    pub doc: Doc,
    pub original_slug: String,
    pub manifest: CentyManifest,
}

/// Create a new doc
pub async fn create_doc(
    project_path: &Path,
    options: CreateDocOptions,
) -> Result<CreateDocResult, DocError> {
    // Validate title
    if options.title.trim().is_empty() {
        return Err(DocError::TitleRequired);
    }

    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");

    // Ensure docs directory exists
    if !docs_path.exists() {
        fs::create_dir_all(&docs_path).await?;
    }

    // Generate or validate slug
    let slug = match options.slug {
        Some(s) if !s.trim().is_empty() => {
            let slug = slugify(&s);
            validate_slug(&slug)?;
            slug
        }
        _ => slugify(&options.title),
    };

    // Check if slug already exists
    let doc_path = docs_path.join(format!("{slug}.md"));
    if doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(slug));
    }

    // Create metadata
    let metadata = DocMetadata::new();

    // Generate doc content with frontmatter
    let doc_content = if let Some(ref template_name) = options.template {
        // Use template engine
        let template_engine = TemplateEngine::new();
        let context = DocTemplateContext {
            title: options.title.clone(),
            content: options.content.clone(),
            slug: slug.clone(),
            created_at: metadata.created_at.clone(),
            updated_at: metadata.updated_at.clone(),
        };
        template_engine
            .render_doc(project_path, template_name, &context)
            .await?
    } else {
        // Use default format
        generate_doc_content(&options.title, &options.content, &metadata)
    };

    // Write the doc file (formatted)
    fs::write(&doc_path, format_markdown(&doc_content)).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let created_file = format!(".centy/docs/{slug}.md");

    Ok(CreateDocResult {
        slug,
        created_file,
        manifest,
    })
}

/// Get a single doc by its slug
pub async fn get_doc(project_path: &Path, slug: &str) -> Result<Doc, DocError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let doc_path = centy_path.join("docs").join(format!("{slug}.md"));

    if !doc_path.exists() {
        return Err(DocError::DocNotFound(slug.to_string()));
    }

    read_doc_from_disk(&doc_path, slug).await
}

/// List all docs
pub async fn list_docs(project_path: &Path) -> Result<Vec<Doc>, DocError> {
    // Check if centy is initialized
    read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");

    if !docs_path.exists() {
        return Ok(Vec::new());
    }

    let mut docs = Vec::new();
    let mut entries = fs::read_dir(&docs_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            if let Some(slug) = path.file_stem().and_then(|s| s.to_str()) {
                // Skip the README.md that's managed by centy
                if slug == "README" {
                    continue;
                }
                if let Ok(doc) = read_doc_from_disk(&path, slug).await {
                    docs.push(doc);
                }
                // Skip docs that can't be read
            }
        }
    }

    // Sort by slug
    docs.sort_by(|a, b| a.slug.cmp(&b.slug));

    Ok(docs)
}

/// Search for docs by slug across all tracked projects
/// This is a global search that doesn't require a project_path
pub async fn get_docs_by_slug(
    slug: &str,
    projects: &[ProjectInfo],
) -> Result<GetDocsBySlugResult, DocError> {
    // Validate slug format
    validate_slug(slug)?;

    let mut found_docs = Vec::new();
    let mut errors = Vec::new();

    for project in projects {
        // Skip uninitialized projects
        if !project.initialized {
            continue;
        }

        let project_path = Path::new(&project.path);

        // Try to get the doc from this project
        match get_doc(project_path, slug).await {
            Ok(doc) => {
                let project_name = project.name.clone().unwrap_or_else(|| {
                    project_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| project.path.clone())
                });

                found_docs.push(DocWithProject {
                    doc,
                    project_path: project.path.clone(),
                    project_name,
                });
            }
            Err(DocError::DocNotFound(_)) => {
                // Not an error - doc simply doesn't exist in this project
            }
            Err(DocError::NotInitialized) => {
                // Skip - project not properly initialized
            }
            Err(e) => {
                // Log non-fatal errors but continue searching
                errors.push(format!("Error searching {}: {}", project.path, e));
            }
        }
    }

    Ok(GetDocsBySlugResult {
        docs: found_docs,
        errors,
    })
}

/// Update an existing doc
pub async fn update_doc(
    project_path: &Path,
    slug: &str,
    options: UpdateDocOptions,
) -> Result<UpdateDocResult, DocError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let docs_path = centy_path.join("docs");
    let doc_path = docs_path.join(format!("{slug}.md"));

    if !doc_path.exists() {
        return Err(DocError::DocNotFound(slug.to_string()));
    }

    // Read current doc
    let current = read_doc_from_disk(&doc_path, slug).await?;

    // Apply updates
    let new_title = options.title.unwrap_or(current.title);
    let new_content = options.content.unwrap_or(current.content);

    // Handle slug rename
    let new_slug = match options.new_slug {
        Some(s) if !s.trim().is_empty() && s != slug => {
            let new_slug = slugify(&s);
            validate_slug(&new_slug)?;

            // Check if new slug already exists
            let new_path = docs_path.join(format!("{new_slug}.md"));
            if new_path.exists() {
                return Err(DocError::SlugAlreadyExists(new_slug));
            }

            Some(new_slug)
        }
        _ => None,
    };

    // Create updated metadata
    let updated_metadata = DocMetadata {
        created_at: current.metadata.created_at.clone(),
        updated_at: now_iso(),
    };

    // Generate updated content
    let doc_content = generate_doc_content(&new_title, &new_content, &updated_metadata);

    // Handle file rename or update
    let final_slug = if let Some(ref new_slug) = new_slug {
        // Remove old file
        fs::remove_file(&doc_path).await?;

        // Write new file
        let new_path = docs_path.join(format!("{new_slug}.md"));
        fs::write(&new_path, &doc_content).await?;

        new_slug.clone()
    } else {
        // Just update the existing file
        fs::write(&doc_path, &doc_content).await?;

        slug.to_string()
    };

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let doc = Doc {
        slug: final_slug,
        title: new_title,
        content: new_content,
        metadata: updated_metadata,
    };

    Ok(UpdateDocResult { doc, manifest })
}

/// Delete a doc
pub async fn delete_doc(project_path: &Path, slug: &str) -> Result<DeleteDocResult, DocError> {
    // Check if centy is initialized
    let mut manifest = read_manifest(project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    let centy_path = get_centy_path(project_path);
    let doc_path = centy_path.join("docs").join(format!("{slug}.md"));

    if !doc_path.exists() {
        return Err(DocError::DocNotFound(slug.to_string()));
    }

    // Remove the file
    fs::remove_file(&doc_path).await?;

    // Update manifest timestamp
    update_manifest_timestamp(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(DeleteDocResult { manifest })
}

/// Move a doc to another project
///
/// The doc is transferred to the target project and deleted from the source.
/// A new slug can be provided if there's a conflict in the target project.
///
/// # Arguments
/// * `options` - Move options specifying source, target, slug, and optional new slug
///
/// # Returns
/// The moved doc with the original slug for reference, plus both manifests
pub async fn move_doc(options: MoveDocOptions) -> Result<MoveDocResult, DocError> {
    // Verify not same project
    if options.source_project_path == options.target_project_path {
        return Err(DocError::SameProjectMove);
    }

    // Validate source project is initialized
    let mut source_manifest = read_manifest(&options.source_project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    // Validate target project is initialized
    let mut target_manifest = read_manifest(&options.target_project_path)
        .await?
        .ok_or(DocError::TargetNotInitialized)?;

    // Read source doc
    let source_centy = get_centy_path(&options.source_project_path);
    let source_doc_path = source_centy.join("docs").join(format!("{}.md", options.slug));

    if !source_doc_path.exists() {
        return Err(DocError::DocNotFound(options.slug.clone()));
    }

    // Validate source doc is readable
    let _source_doc = read_doc_from_disk(&source_doc_path, &options.slug).await?;

    // Determine target slug
    let target_slug = match options.new_slug {
        Some(ref s) if !s.trim().is_empty() => {
            let slug = slugify(s);
            validate_slug(&slug)?;
            slug
        }
        _ => options.slug.clone(),
    };

    // Check for conflict in target project
    let target_centy = get_centy_path(&options.target_project_path);
    let target_docs_path = target_centy.join("docs");
    fs::create_dir_all(&target_docs_path).await?;
    let target_doc_path = target_docs_path.join(format!("{target_slug}.md"));

    if target_doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(target_slug));
    }

    // Copy file to target (preserving metadata)
    fs::copy(&source_doc_path, &target_doc_path).await?;

    // If slug changed, update the content to reflect new title header
    if target_slug != options.slug {
        // Re-read and re-write with potentially updated metadata
        let content = fs::read_to_string(&target_doc_path).await?;
        let (title, body, mut metadata) = parse_doc_content(&content);
        metadata.updated_at = now_iso();
        let new_content = generate_doc_content(&title, &body, &metadata);
        fs::write(&target_doc_path, new_content).await?;
    }

    // Delete from source project
    fs::remove_file(&source_doc_path).await?;

    // Update both manifests
    update_manifest_timestamp(&mut source_manifest);
    update_manifest_timestamp(&mut target_manifest);
    write_manifest(&options.source_project_path, &source_manifest).await?;
    write_manifest(&options.target_project_path, &target_manifest).await?;

    // Read the moved doc
    let moved_doc = read_doc_from_disk(&target_doc_path, &target_slug).await?;

    Ok(MoveDocResult {
        doc: moved_doc,
        old_slug: options.slug,
        source_manifest,
        target_manifest,
    })
}

/// Duplicate a doc to the same or different project
///
/// Creates a copy of the doc with a new slug.
///
/// # Arguments
/// * `options` - Duplicate options specifying source, target, slug, optional new slug, and optional new title
///
/// # Returns
/// The new duplicate doc with the original slug for reference
pub async fn duplicate_doc(options: DuplicateDocOptions) -> Result<DuplicateDocResult, DocError> {
    // Validate source project is initialized
    read_manifest(&options.source_project_path)
        .await?
        .ok_or(DocError::NotInitialized)?;

    // Validate target project is initialized
    let mut target_manifest = read_manifest(&options.target_project_path)
        .await?
        .ok_or(DocError::TargetNotInitialized)?;

    // Read source doc
    let source_centy = get_centy_path(&options.source_project_path);
    let source_doc_path = source_centy.join("docs").join(format!("{}.md", options.slug));

    if !source_doc_path.exists() {
        return Err(DocError::DocNotFound(options.slug.clone()));
    }

    let source_doc = read_doc_from_disk(&source_doc_path, &options.slug).await?;

    // Determine new slug
    let new_slug = match options.new_slug {
        Some(ref s) if !s.trim().is_empty() => {
            let slug = slugify(s);
            validate_slug(&slug)?;
            slug
        }
        _ => format!("{}-copy", options.slug),
    };

    // Validate new slug
    validate_slug(&new_slug)?;

    // Check for conflict in target project
    let target_centy = get_centy_path(&options.target_project_path);
    let target_docs_path = target_centy.join("docs");
    fs::create_dir_all(&target_docs_path).await?;
    let target_doc_path = target_docs_path.join(format!("{new_slug}.md"));

    if target_doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(new_slug));
    }

    // Prepare new title
    let new_title = options.new_title.unwrap_or_else(|| {
        format!("Copy of {}", source_doc.title)
    });

    // Create new metadata with fresh timestamps
    let new_metadata = DocMetadata::new();

    // Generate new doc content
    let doc_content = generate_doc_content(&new_title, &source_doc.content, &new_metadata);
    fs::write(&target_doc_path, &doc_content).await?;

    // Update target manifest
    update_manifest_timestamp(&mut target_manifest);
    write_manifest(&options.target_project_path, &target_manifest).await?;

    // Read the new doc
    let new_doc = read_doc_from_disk(&target_doc_path, &new_slug).await?;

    Ok(DuplicateDocResult {
        doc: new_doc,
        original_slug: options.slug,
        manifest: target_manifest,
    })
}

/// Read a doc from disk
async fn read_doc_from_disk(doc_path: &Path, slug: &str) -> Result<Doc, DocError> {
    let content = fs::read_to_string(doc_path).await?;
    let (title, body, metadata) = parse_doc_content(&content);

    Ok(Doc {
        slug: slug.to_string(),
        title,
        content: body,
        metadata,
    })
}

/// Generate doc content with YAML frontmatter
fn generate_doc_content(title: &str, content: &str, metadata: &DocMetadata) -> String {
    format!(
        "---\ntitle: \"{}\"\ncreatedAt: \"{}\"\nupdatedAt: \"{}\"\n---\n\n# {}\n\n{}",
        escape_yaml_string(title),
        metadata.created_at,
        metadata.updated_at,
        title,
        content
    )
}

/// Parse doc content extracting title, body, and metadata from frontmatter
fn parse_doc_content(content: &str) -> (String, String, DocMetadata) {
    let lines: Vec<&str> = content.lines().collect();

    // Check for frontmatter
    if lines.first() == Some(&"---") {
        // Find closing ---
        if let Some(end_idx) = lines.iter().skip(1).position(|&line| line == "---") {
            let frontmatter: Vec<&str> = lines[1..=end_idx].to_vec();
            let body_start = end_idx + 2;

            // Parse frontmatter
            let mut title = String::new();
            let mut created_at = String::new();
            let mut updated_at = String::new();

            for line in frontmatter {
                if let Some(value) = line.strip_prefix("title:") {
                    title = value.trim().trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("createdAt:") {
                    created_at = value.trim().trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("updatedAt:") {
                    updated_at = value.trim().trim_matches('"').to_string();
                }
            }

            // Get body (skip empty lines after frontmatter)
            let body_lines: Vec<&str> = lines[body_start..]
                .iter()
                .skip_while(|line| line.is_empty())
                .copied()
                .collect();

            // Skip the title line if it matches (# Title)
            let body = if body_lines.first().is_some_and(|l| l.starts_with("# ")) {
                body_lines[1..]
                    .iter()
                    .skip_while(|line| line.is_empty())
                    .copied()
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                body_lines.join("\n")
            };

            let metadata = DocMetadata {
                created_at: if created_at.is_empty() { now_iso() } else { created_at },
                updated_at: if updated_at.is_empty() { now_iso() } else { updated_at },
            };

            return (title, body.trim_end().to_string(), metadata);
        }
    }

    // No frontmatter - extract title from first # heading
    let mut title = String::new();
    let mut body_start = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("# ") {
            title = line.strip_prefix("# ").unwrap_or("").to_string();
            body_start = i + 1;
            break;
        }
    }

    let body = lines[body_start..]
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string();

    (title, body, DocMetadata::new())
}

/// Convert a string to a URL-friendly slug
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c == ' ' || c == '_' || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Validate a slug
fn validate_slug(slug: &str) -> Result<(), DocError> {
    if slug.is_empty() {
        return Err(DocError::InvalidSlug("Slug cannot be empty".to_string()));
    }

    if !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(DocError::InvalidSlug(
            "Slug can only contain alphanumeric characters and hyphens".to_string(),
        ));
    }

    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(DocError::InvalidSlug(
            "Slug cannot start or end with a hyphen".to_string(),
        ));
    }

    Ok(())
}

/// Escape special characters in YAML strings
fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Getting Started Guide"), "getting-started-guide");
        assert_eq!(slugify("API v2"), "api-v2");
        assert_eq!(slugify("  Spaces  "), "spaces");
        assert_eq!(slugify("multiple---hyphens"), "multiple-hyphens");
        assert_eq!(slugify("Under_score"), "under-score");
    }

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("hello-world").is_ok());
        assert!(validate_slug("api-v2").is_ok());
        assert!(validate_slug("").is_err());
        assert!(validate_slug("-start").is_err());
        assert!(validate_slug("end-").is_err());
        assert!(validate_slug("has space").is_err());
    }

    #[test]
    fn test_parse_doc_content_with_frontmatter() {
        let content = r#"---
title: "My Doc"
createdAt: "2024-01-01T00:00:00Z"
updatedAt: "2024-01-02T00:00:00Z"
---

# My Doc

This is the content."#;

        let (title, body, metadata) = parse_doc_content(content);
        assert_eq!(title, "My Doc");
        assert_eq!(body, "This is the content.");
        assert_eq!(metadata.created_at, "2024-01-01T00:00:00Z");
        assert_eq!(metadata.updated_at, "2024-01-02T00:00:00Z");
    }

    #[test]
    fn test_parse_doc_content_without_frontmatter() {
        let content = "# Simple Doc\n\nJust some content here.";
        let (title, body, _metadata) = parse_doc_content(content);
        assert_eq!(title, "Simple Doc");
        assert_eq!(body, "Just some content here.");
    }

    #[test]
    fn test_generate_doc_content() {
        let metadata = DocMetadata {
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-02T00:00:00Z".to_string(),
        };
        let content = generate_doc_content("Test Title", "Body text", &metadata);

        assert!(content.contains("title: \"Test Title\""));
        assert!(content.contains("# Test Title"));
        assert!(content.contains("Body text"));
    }

    #[test]
    fn test_escape_yaml_string() {
        assert_eq!(escape_yaml_string("simple"), "simple");
        assert_eq!(escape_yaml_string("with \"quotes\""), "with \\\"quotes\\\"");
        assert_eq!(escape_yaml_string("back\\slash"), "back\\\\slash");
    }
}
