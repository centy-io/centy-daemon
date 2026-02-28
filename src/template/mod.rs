mod engine;
mod types;

pub use engine::{TemplateEngine, TemplateError};
// TemplateType is used in engine.rs internally and exported from lib.rs for external consumers
#[allow(unused_imports)]
pub use types::{IssueTemplateContext, TemplateType};
