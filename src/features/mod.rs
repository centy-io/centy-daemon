mod crud;
mod instruction;
mod types;

// Feature module exports - WIP functionality for issue compaction
#[allow(unused_imports)]
#[allow(dead_code)] // Part of WIP features module
pub use crud::{
    build_compacted_refs, get_compact, get_feature_status, get_instruction,
    list_uncompacted_issues, mark_issues_compacted, save_migration, update_compact, FeatureError,
};
#[allow(unused_imports)]
pub use instruction::DEFAULT_INSTRUCTION_CONTENT;
#[allow(unused_imports)]
#[allow(dead_code)] // Part of WIP features module
pub use types::{CompactedIssueRef, FeatureStatus};
