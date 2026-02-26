/// Returned by API (enriched with live data from disk)
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub path: String,
    pub first_accessed: String,
    pub last_accessed: String,
    pub issue_count: u32,
    pub doc_count: u32,
    pub initialized: bool,
    pub name: Option<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub organization_slug: Option<String>,
    pub organization_name: Option<String>,
    pub user_title: Option<String>,
    pub project_title: Option<String>,
    pub project_version: Option<String>,
    pub project_behind: bool,
}
/// Organization info returned by API (enriched with project count)
#[derive(Debug, Clone)]
pub struct OrganizationInfo {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub project_count: u32,
}
/// Options for listing projects
#[derive(Debug, Clone, Default)]
pub struct ListProjectsOptions<'a> {
    pub include_stale: bool,
    pub include_uninitialized: bool,
    pub include_archived: bool,
    pub organization_slug: Option<&'a str>,
    pub ungrouped_only: bool,
    pub include_temp: bool,
}
