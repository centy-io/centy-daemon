use crate::manifest::{read_manifest, update_manifest, write_manifest, CentyManifest};
use crate::registry::{get_org_projects, get_project_info, ProjectInfo};
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

    #[error("Doc '{0}' is not soft-deleted")]
    DocNotDeleted(String),

    #[error("Doc '{0}' is already soft-deleted")]
    DocAlreadyDeleted(String),

    #[error("Template error: {0}")]
    TemplateError(#[from] TemplateError),

    #[error("Target project not initialized")]
    TargetNotInitialized,

    #[error("Cannot move doc to same project")]
    SameProjectMove,

    #[error("Cannot create org doc: project has no organization")]
    NoOrganization,

    #[error("Registry error: {0}")]
    RegistryError(String),
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
    /// ISO timestamp when soft-deleted (None if not deleted)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    /// Whether this doc is organization-level (synced on creation)
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_org_doc: bool,
    /// Organization slug for org docs (for traceability)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_slug: Option<String>,
}

impl DocMetadata {
    #[must_use]
    pub fn new() -> Self {
        let now = now_iso();
        Self {
            created_at: now.clone(),
            updated_at: now,
            deleted_at: None,
            is_org_doc: false,
            org_slug: None,
        }
    }

    #[must_use]
    pub fn new_org_doc(org_slug: &str) -> Self {
        let now = now_iso();
        Self {
            created_at: now.clone(),
            updated_at: now,
            deleted_at: None,
            is_org_doc: true,
            org_slug: Some(org_slug.to_string()),
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
    /// Create as organization-wide doc (syncs to all org projects)
    pub is_org_doc: bool,
}

/// Result of syncing an org doc to another project
#[derive(Debug, Clone)]
pub struct OrgDocSyncResult {
    pub project_path: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Result of doc creation
#[derive(Debug, Clone)]
pub struct CreateDocResult {
    pub slug: String,
    pub created_file: String,
    pub manifest: CentyManifest,
    /// Results from syncing to other org projects (empty for non-org docs)
    pub sync_results: Vec<OrgDocSyncResult>,
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
    /// Results from syncing to other org projects (empty for non-org docs)
    pub sync_results: Vec<OrgDocSyncResult>,
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

    // Get organization info if this is an org doc
    let org_slug = if options.is_org_doc {
        // Get project's organization
        let project_path_str = project_path.to_string_lossy().to_string();
        let project_info = get_project_info(&project_path_str)
            .await
            .map_err(|e| DocError::RegistryError(e.to_string()))?;

        match project_info.and_then(|p| p.organization_slug) {
            Some(slug) => Some(slug),
            None => return Err(DocError::NoOrganization),
        }
    } else {
        None
    };

    // Create metadata (with or without org info)
    let metadata = if let Some(ref org) = org_slug {
        DocMetadata::new_org_doc(org)
    } else {
        DocMetadata::new()
    };

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
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let created_file = format!(".centy/docs/{slug}.md");

    // Sync to other org projects if this is an org doc
    let sync_results = if let Some(ref org) = org_slug {
        sync_org_doc_to_projects(org, project_path, &slug, &options.title, &options.content).await
    } else {
        Vec::new()
    };

    Ok(CreateDocResult {
        slug,
        created_file,
        manifest,
        sync_results,
    })
}

/// Sync an org doc to all other projects in the organization
async fn sync_org_doc_to_projects(
    org_slug: &str,
    source_project_path: &Path,
    slug: &str,
    title: &str,
    content: &str,
) -> Vec<OrgDocSyncResult> {
    let source_path_str = source_project_path.to_string_lossy().to_string();

    // Get all other projects in the org
    let org_projects = match get_org_projects(org_slug, Some(&source_path_str)).await {
        Ok(projects) => projects,
        Err(e) => {
            // Return a single error result
            return vec![OrgDocSyncResult {
                project_path: "<registry>".to_string(),
                success: false,
                error: Some(format!("Failed to get org projects: {e}")),
            }];
        }
    };

    let mut results = Vec::new();

    for project in org_projects {
        let target_path = Path::new(&project.path);
        let result = create_doc_in_project(target_path, slug, title, content, org_slug).await;

        results.push(OrgDocSyncResult {
            project_path: project.path.clone(),
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
        });
    }

    results
}

/// Create a doc in a specific project (used for org doc sync)
/// This is a simpler version that doesn't do org sync recursion
async fn create_doc_in_project(
    project_path: &Path,
    slug: &str,
    title: &str,
    content: &str,
    org_slug: &str,
) -> Result<(), DocError> {
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

    let doc_path = docs_path.join(format!("{slug}.md"));

    // Skip if already exists (don't overwrite)
    if doc_path.exists() {
        return Err(DocError::SlugAlreadyExists(slug.to_string()));
    }

    // Create org doc metadata
    let metadata = DocMetadata::new_org_doc(org_slug);

    // Generate doc content
    let doc_content = generate_doc_content(title, content, &metadata);

    // Write the doc file (formatted)
    fs::write(&doc_path, format_markdown(&doc_content)).await?;

    // Update manifest timestamp
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(())
}

/// Sync an org doc update to all other projects in the organization
/// Creates the doc if it doesn't exist in a project, or updates it if it does
async fn sync_org_doc_update_to_projects(
    org_slug: &str,
    source_project_path: &Path,
    slug: &str,
    title: &str,
    content: &str,
    old_slug: Option<&str>,
) -> Vec<OrgDocSyncResult> {
    let source_path_str = source_project_path.to_string_lossy().to_string();

    // Get all other projects in the org
    let org_projects = match get_org_projects(org_slug, Some(&source_path_str)).await {
        Ok(projects) => projects,
        Err(e) => {
            // Return a single error result
            return vec![OrgDocSyncResult {
                project_path: "<registry>".to_string(),
                success: false,
                error: Some(format!("Failed to get org projects: {e}")),
            }];
        }
    };

    let mut results = Vec::new();

    for project in org_projects {
        let target_path = Path::new(&project.path);
        let result =
            update_or_create_doc_in_project(target_path, slug, title, content, org_slug, old_slug)
                .await;

        results.push(OrgDocSyncResult {
            project_path: project.path.clone(),
            success: result.is_ok(),
            error: result.err().map(|e| e.to_string()),
        });
    }

    results
}

/// Update or create a doc in a specific project (used for org doc sync on update)
/// If the doc exists, updates it. If it doesn't exist, creates it.
/// If old_slug is provided (slug rename), deletes the old doc and creates new.
async fn update_or_create_doc_in_project(
    project_path: &Path,
    slug: &str,
    title: &str,
    content: &str,
    org_slug: &str,
    old_slug: Option<&str>,
) -> Result<(), DocError> {
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

    let doc_path = docs_path.join(format!("{slug}.md"));

    // Handle slug rename: delete old file if it exists
    if let Some(old) = old_slug {
        if old != slug {
            let old_doc_path = docs_path.join(format!("{old}.md"));
            if old_doc_path.exists() {
                fs::remove_file(&old_doc_path).await?;
            }
        }
    }

    // Check if doc already exists (to preserve created_at)
    let metadata = if doc_path.exists() {
        // Read existing to preserve created_at
        let existing = read_doc_from_disk(&doc_path, slug).await?;
        DocMetadata {
            created_at: existing.metadata.created_at,
            updated_at: now_iso(),
            deleted_at: None, // Clear any soft-delete on update
            is_org_doc: true,
            org_slug: Some(org_slug.to_string()),
        }
    } else {
        // Check if there's an old slug doc to get created_at from
        let old_created_at = if let Some(old) = old_slug {
            let old_doc_path = docs_path.join(format!("{old}.md"));
            if old_doc_path.exists() {
                read_doc_from_disk(&old_doc_path, old)
                    .await
                    .ok()
                    .map(|d| d.metadata.created_at)
            } else {
                None
            }
        } else {
            None
        };

        DocMetadata {
            created_at: old_created_at.unwrap_or_else(now_iso),
            updated_at: now_iso(),
            deleted_at: None,
            is_org_doc: true,
            org_slug: Some(org_slug.to_string()),
        }
    };

    // Generate doc content
    let doc_content = generate_doc_content(title, content, &metadata);

    // Write the doc file (formatted)
    fs::write(&doc_path, format_markdown(&doc_content)).await?;

    // Update manifest timestamp
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    Ok(())
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
pub async fn list_docs(project_path: &Path, include_deleted: bool) -> Result<Vec<Doc>, DocError> {
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
                    // Filter out soft-deleted unless include_deleted is true
                    if include_deleted || doc.metadata.deleted_at.is_none() {
                        docs.push(doc);
                    }
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

    // Create updated metadata (preserve org doc fields)
    let updated_metadata = DocMetadata {
        created_at: current.metadata.created_at.clone(),
        updated_at: now_iso(),
        deleted_at: current.metadata.deleted_at.clone(),
        is_org_doc: current.metadata.is_org_doc,
        org_slug: current.metadata.org_slug.clone(),
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
    update_manifest(&mut manifest);
    write_manifest(project_path, &manifest).await?;

    let doc = Doc {
        slug: final_slug.clone(),
        title: new_title.clone(),
        content: new_content.clone(),
        metadata: updated_metadata,
    };

    // Sync to other org projects if this is an org doc
    let sync_results = if doc.metadata.is_org_doc {
        if let Some(ref org) = doc.metadata.org_slug {
            // Determine the old slug for rename handling
            let old_slug_for_sync = new_slug.as_ref().map(|_| slug);
            sync_org_doc_update_to_projects(
                org,
                project_path,
                &final_slug,
                &new_title,
                &new_content,
                old_slug_for_sync,
            )
            .await
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(UpdateDocResult {
        doc,
        manifest,
        sync_results,
    })
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
    let source_doc_path = source_centy
        .join("docs")
        .join(format!("{}.md", options.slug));

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
    update_manifest(&mut source_manifest);
    update_manifest(&mut target_manifest);
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
    let source_doc_path = source_centy
        .join("docs")
        .join(format!("{}.md", options.slug));

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
    let new_title = options
        .new_title
        .unwrap_or_else(|| format!("Copy of {}", source_doc.title));

    // Create new metadata with fresh timestamps
    let new_metadata = DocMetadata::new();

    // Generate new doc content
    let doc_content = generate_doc_content(&new_title, &source_doc.content, &new_metadata);
    fs::write(&target_doc_path, &doc_content).await?;

    // Update target manifest
    update_manifest(&mut target_manifest);
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
    let deleted_line = metadata
        .deleted_at
        .as_ref()
        .map(|d| format!("\ndeletedAt: \"{d}\""))
        .unwrap_or_default();
    let org_doc_line = if metadata.is_org_doc {
        "\nisOrgDoc: true".to_string()
    } else {
        String::new()
    };
    let org_slug_line = metadata
        .org_slug
        .as_ref()
        .map(|s| format!("\norgSlug: \"{s}\""))
        .unwrap_or_default();
    format!(
        "---\ntitle: \"{}\"\ncreatedAt: \"{}\"\nupdatedAt: \"{}\"{}{}{}\n---\n\n# {}\n\n{}",
        escape_yaml_string(title),
        metadata.created_at,
        metadata.updated_at,
        deleted_line,
        org_doc_line,
        org_slug_line,
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
            let frontmatter: Vec<&str> = lines.get(1..=end_idx).unwrap_or(&[]).to_vec();
            let body_start = end_idx.saturating_add(2);

            // Parse frontmatter
            let mut title = String::new();
            let mut created_at = String::new();
            let mut updated_at = String::new();
            let mut deleted_at: Option<String> = None;
            let mut is_org_doc = false;
            let mut org_slug: Option<String> = None;

            for line in frontmatter {
                if let Some(value) = line.strip_prefix("title:") {
                    title = value.trim().trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("createdAt:") {
                    created_at = value.trim().trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("updatedAt:") {
                    updated_at = value.trim().trim_matches('"').to_string();
                } else if let Some(value) = line.strip_prefix("deletedAt:") {
                    let v = value.trim().trim_matches('"').to_string();
                    if !v.is_empty() {
                        deleted_at = Some(v);
                    }
                } else if let Some(value) = line.strip_prefix("isOrgDoc:") {
                    is_org_doc = value.trim() == "true";
                } else if let Some(value) = line.strip_prefix("orgSlug:") {
                    let v = value.trim().trim_matches('"').to_string();
                    if !v.is_empty() {
                        org_slug = Some(v);
                    }
                }
            }

            // Get body (skip empty lines after frontmatter)
            let body_lines: Vec<&str> = lines
                .get(body_start..)
                .unwrap_or(&[])
                .iter()
                .skip_while(|line| line.is_empty())
                .copied()
                .collect();

            // Skip the title line if it matches (# Title)
            let body = if body_lines.first().is_some_and(|l| l.starts_with("# ")) {
                body_lines
                    .get(1..)
                    .unwrap_or(&[])
                    .iter()
                    .skip_while(|line| line.is_empty())
                    .copied()
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                body_lines.join("\n")
            };

            let metadata = DocMetadata {
                created_at: if created_at.is_empty() {
                    now_iso()
                } else {
                    created_at
                },
                updated_at: if updated_at.is_empty() {
                    now_iso()
                } else {
                    updated_at
                },
                deleted_at,
                is_org_doc,
                org_slug,
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
            body_start = i.saturating_add(1);
            break;
        }
    }

    let body = lines
        .get(body_start..)
        .unwrap_or(&[])
        .iter()
        .skip_while(|line| line.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string();

    (
        title,
        body,
        DocMetadata {
            created_at: now_iso(),
            updated_at: now_iso(),
            deleted_at: None,
            is_org_doc: false,
            org_slug: None,
        },
    )
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
            deleted_at: None,
            is_org_doc: false,
            org_slug: None,
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
