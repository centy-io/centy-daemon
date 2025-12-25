use std::cmp::Ordering;
use std::path::Path;

use crate::issue::{list_issues, Issue};
use crate::registry::{list_projects, ListProjectsOptions};
use crate::utils::format_display_path;

use super::ast::Query;
use super::error::SearchError;
use super::evaluator::evaluate;
use super::parser::{format_query, parse_query};

/// Options for executing a search
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// The query string to parse and execute
    pub query: String,
    /// Optional sorting
    pub sort: Option<SortOptions>,
    /// Whether to search across all tracked projects
    pub multi_project: bool,
    /// Project path (required if multi_project is false)
    pub project_path: Option<String>,
}

/// Sorting options
#[derive(Debug, Clone)]
pub struct SortOptions {
    /// Field to sort by
    pub field: SortField,
    /// Sort descending (default is ascending)
    pub descending: bool,
}

/// Fields that can be sorted by
#[derive(Debug, Clone, PartialEq)]
pub enum SortField {
    Title,
    Status,
    Priority,
    DisplayNumber,
    CreatedAt,
    UpdatedAt,
    Custom(String),
}

impl std::str::FromStr for SortField {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "title" => SortField::Title,
            "status" => SortField::Status,
            "priority" | "prio" | "p" => SortField::Priority,
            "displaynumber" | "number" | "num" | "n" => SortField::DisplayNumber,
            "createdat" | "created" => SortField::CreatedAt,
            "updatedat" | "updated" => SortField::UpdatedAt,
            other => SortField::Custom(other.to_string()),
        })
    }
}

/// Result of a search operation
#[derive(Debug)]
pub struct SearchResult {
    /// Matching issues with project info
    pub results: Vec<SearchResultIssue>,
    /// Total count of results
    pub total_count: usize,
    /// Debug: the parsed query representation
    pub parsed_query: String,
}

/// A search result item containing issue and project info
#[derive(Debug)]
pub struct SearchResultIssue {
    pub issue: Issue,
    pub project_path: String,
    pub project_name: String,
    pub display_path: String,
}

/// Execute an advanced search
pub async fn advanced_search(options: SearchOptions) -> Result<SearchResult, SearchError> {
    // Parse the query
    let query = parse_query(&options.query)?;
    let parsed_query = query.as_ref().map(format_query).unwrap_or_default();

    // Collect issues
    let mut results = if options.multi_project {
        search_all_projects(query.as_ref()).await?
    } else {
        let project_path = options.project_path.ok_or_else(|| {
            SearchError::ParseError("project_path required for single-project search".to_string())
        })?;
        search_single_project(&project_path, query.as_ref()).await?
    };

    // Sort results
    if let Some(sort) = &options.sort {
        sort_results(&mut results, sort);
    }

    let total_count = results.len();

    Ok(SearchResult {
        results,
        total_count,
        parsed_query,
    })
}

async fn search_single_project(
    project_path: &str,
    query: Option<&Query>,
) -> Result<Vec<SearchResultIssue>, SearchError> {
    let path = Path::new(project_path);

    // Get project name from path
    let project_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // List all issues (no filters, we'll apply our own)
    let issues = list_issues(path, None, None, None, false)
        .await
        .map_err(|e| SearchError::IssueError(e.to_string()))?;

    // Filter issues
    let filtered: Vec<SearchResultIssue> = issues
        .into_iter()
        .filter(|issue| {
            match query {
                Some(q) => evaluate(q, issue),
                None => true, // No query means match all
            }
        })
        .map(|issue| SearchResultIssue {
            issue,
            project_path: project_path.to_string(),
            project_name: project_name.clone(),
            display_path: format_display_path(project_path),
        })
        .collect();

    Ok(filtered)
}

async fn search_all_projects(query: Option<&Query>) -> Result<Vec<SearchResultIssue>, SearchError> {
    // Get all tracked projects
    let projects = list_projects(ListProjectsOptions::default())
        .await
        .map_err(|e| SearchError::RegistryError(e.to_string()))?;

    let mut all_results = Vec::new();

    // Search each project
    for project in projects {
        match search_single_project(&project.path, query).await {
            Ok(results) => all_results.extend(results),
            Err(e) => {
                // Log error but continue with other projects
                tracing::warn!("Error searching project {}: {}", project.path, e);
            }
        }
    }

    Ok(all_results)
}

fn sort_results(results: &mut [SearchResultIssue], sort: &SortOptions) {
    results.sort_by(|a, b| {
        let cmp = compare_by_field(&a.issue, &b.issue, &sort.field);
        if sort.descending {
            cmp.reverse()
        } else {
            cmp
        }
    });
}

fn compare_by_field(a: &Issue, b: &Issue, field: &SortField) -> Ordering {
    match field {
        SortField::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
        SortField::Status => a.metadata.status.to_lowercase().cmp(&b.metadata.status.to_lowercase()),
        SortField::Priority => a.metadata.priority.cmp(&b.metadata.priority),
        SortField::DisplayNumber => a.metadata.display_number.cmp(&b.metadata.display_number),
        SortField::CreatedAt => a.metadata.created_at.cmp(&b.metadata.created_at),
        SortField::UpdatedAt => a.metadata.updated_at.cmp(&b.metadata.updated_at),
        SortField::Custom(name) => {
            let a_val = a.metadata.custom_fields.get(name).map(String::as_str).unwrap_or("");
            let b_val = b.metadata.custom_fields.get(name).map(String::as_str).unwrap_or("");
            a_val.to_lowercase().cmp(&b_val.to_lowercase())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_field_from_str() {
        assert_eq!("title".parse::<SortField>().unwrap(), SortField::Title);
        assert_eq!("priority".parse::<SortField>().unwrap(), SortField::Priority);
        assert_eq!("prio".parse::<SortField>().unwrap(), SortField::Priority);
        assert_eq!("createdAt".parse::<SortField>().unwrap(), SortField::CreatedAt);
        assert_eq!("custom".parse::<SortField>().unwrap(), SortField::Custom("custom".to_string()));
    }
}
