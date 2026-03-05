mod frontmatter;
mod issue_meta;
pub use frontmatter::IssueFrontmatter;
pub use issue_meta::IssueMetadata;
pub use std::collections::HashMap;
#[cfg(test)]
#[path = "../metadata_tests.rs"]
mod metadata_tests;
