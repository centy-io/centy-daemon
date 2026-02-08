use crate::registry::track_project_async;
use crate::search::{SearchOptions, SortOptions};
use crate::server::convert_entity::issue_to_proto;
use crate::server::proto::{
    AdvancedSearchRequest, AdvancedSearchResponse, SearchResultIssue as ProtoSearchResultIssue,
};
use tonic::{Response, Status};

pub async fn advanced_search(
    req: AdvancedSearchRequest,
) -> Result<Response<AdvancedSearchResponse>, Status> {
    // Track project if single-project search
    if !req.multi_project && !req.project_path.is_empty() {
        track_project_async(req.project_path.clone());
    }

    // Parse sort options
    let sort = if req.sort_by.is_empty() {
        None
    } else {
        Some(SortOptions {
            field: req.sort_by.parse().unwrap(),
            descending: req.sort_descending,
        })
    };

    let options = SearchOptions {
        query: req.query,
        sort,
        multi_project: req.multi_project,
        project_path: if req.project_path.is_empty() {
            None
        } else {
            Some(req.project_path)
        },
    };

    match crate::search::advanced_search(options).await {
        Ok(result) => {
            // Convert results to proto types
            let results: Vec<ProtoSearchResultIssue> = result
                .results
                .into_iter()
                .map(|r| {
                    // Use default priority_levels since we can't do async in map
                    let priority_levels = 3;
                    ProtoSearchResultIssue {
                        issue: Some(issue_to_proto(&r.issue, priority_levels)),
                        project_path: r.project_path,
                        project_name: r.project_name,
                        display_path: r.display_path,
                    }
                })
                .collect();

            let total_count = results.len() as i32;

            Ok(Response::new(AdvancedSearchResponse {
                success: true,
                error: String::new(),
                results,
                total_count,
                parsed_query: result.parsed_query,
            }))
        }
        Err(e) => Ok(Response::new(AdvancedSearchResponse {
            success: false,
            error: e.to_string(),
            results: vec![],
            total_count: 0,
            parsed_query: String::new(),
        })),
    }
}
