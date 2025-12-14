mod ast;
mod error;
mod evaluator;
mod executor;
mod parser;

// Re-export public API
// These are used by the library crate (lib.rs) even if the binary doesn't use them directly
#[allow(unused_imports)]
pub use error::SearchError;
#[allow(unused_imports)]
pub use executor::{advanced_search, SearchOptions, SearchResult, SearchResultIssue, SortField, SortOptions};
#[allow(unused_imports)]
pub use parser::{format_query, parse_query};
