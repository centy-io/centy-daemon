use std::path::Path;
use thiserror::Error;

use crate::item::entities::issue::{priority_label, Issue};
use crate::template::{LlmTemplateContext, TemplateEngine, TemplateError, TemplateType};

#[derive(Error, Debug)]
pub enum PromptError {
    #[error("Template error: {0}")]
    TemplateError(#[from] TemplateError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Action type for prompt building
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmAction {
    Plan,
    Implement,
    Deepdive,
}

impl LlmAction {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            LlmAction::Plan => "plan",
            LlmAction::Implement => "implement",
            LlmAction::Deepdive => "deepdive",
        }
    }

    /// Convert from proto enum value
    #[must_use]
    pub fn from_proto(value: i32) -> Option<Self> {
        match value {
            1 => Some(LlmAction::Plan),
            2 => Some(LlmAction::Implement),
            3 => Some(LlmAction::Deepdive),
            _ => None,
        }
    }

    /// Convert to proto enum value
    #[cfg(test)]
    pub fn to_proto(&self) -> i32 {
        match self {
            LlmAction::Plan => 1,
            LlmAction::Implement => 2,
            LlmAction::Deepdive => 3,
        }
    }
}

/// Base system prompt for LLM integration
pub const BASE_SYSTEM_PROMPT: &str = r"# Issue Context

You are working on an issue from a local issue tracker.

## Getting Started

1. **Explore the project**: First, understand the project structure by exploring the codebase
2. **Understand the CLI**: Check for available CLI commands (look for help flags like `--help` or `-h`)
3. **Review related files**: Look for any configuration, documentation, or related code

## Working Guidelines

- Explore and understand before making changes
- Use the project's CLI tools when available for status updates
- Keep changes focused on the issue at hand

---

";

/// Base prompt specifically for "plan" action
pub const PLAN_ACTION_PROMPT: &str = r"## Your Task: Planning

You are creating an implementation plan for this issue. Your plan should:

1. **Analyze Requirements**: Break down the issue into specific, actionable tasks
2. **Identify Dependencies**: Note any prerequisites or blocking factors
3. **Consider Edge Cases**: Think about potential issues and how to handle them
4. **Suggest Testing Strategy**: Outline how the implementation should be tested

Do NOT implement anything yet. Focus on creating a clear, comprehensive plan that another developer (or you in a subsequent session) can follow.

## Saving Your Plan

After creating your plan, save it using the centy CLI:

```bash
centy add plan <ISSUE_ID> --file <path-to-plan-file>
```

Use the issue ID from the Issue Details section below (either the UUID or the display number like `1`, `2`, etc.)

---

";

/// Base prompt specifically for "implement" action
pub const IMPLEMENT_ACTION_PROMPT: &str = r"## Your Task: Implementation

You are implementing a solution for this issue. You should:

1. **Follow Best Practices**: Write clean, maintainable code
2. **Add Tests**: Include appropriate unit and integration tests
3. **Update Documentation**: Keep docs in sync with code changes
4. **Commit Atomically**: Make small, focused commits with clear messages
5. **Update Issue Status**: Mark the issue as in-progress when starting

When complete, update the issue status to closed.

---

";

/// Base prompt specifically for "deepdive" action
pub const DEEPDIVE_ACTION_PROMPT: &str = r"## Your Task: Deep Dive Brainstorming

Generate comprehensive questions for brainstorming about this issue. Your questions should help explore:

1. **Requirements & Scope**: What does the user really need? What are the edge cases? What are the acceptance criteria?
2. **Technical Considerations**: Architecture decisions, dependencies, performance implications, implementation choices
3. **Risks & Challenges**: What could go wrong? What are the unknowns? What might block progress?

## Output Format

Save your questions to `q&a.md` in this format:

```markdown
# Deep Dive Q&A: [Issue Title]

## Requirements & Scope
- Q: [Question about requirements]
  A:

- Q: [Another question]
  A:

## Technical Considerations
- Q: [Technical question]
  A:

## Risks & Edge Cases
- Q: [Risk-related question]
  A:
```

After creating the file, save it using:
```bash
centy add deepdive <ISSUE_ID> --file q&a.md
rm q&a.md
```

Use the issue ID from the Issue Details section below (either the UUID or the display number like `1`, `2`, etc.)

---

";

/// Build a complete prompt for an LLM agent
pub struct PromptBuilder {
    template_engine: TemplateEngine,
}

impl PromptBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
        }
    }

    /// Build the full prompt for an issue and action
    pub async fn build_prompt(
        &self,
        project_path: &Path,
        issue: &Issue,
        action: LlmAction,
        user_template: Option<&str>,
        priority_levels: u32,
    ) -> Result<String, PromptError> {
        let mut prompt = String::new();

        // 1. Base system prompt
        prompt.push_str(BASE_SYSTEM_PROMPT);

        // 2. Action-specific base prompt
        match action {
            LlmAction::Plan => prompt.push_str(PLAN_ACTION_PROMPT),
            LlmAction::Implement => prompt.push_str(IMPLEMENT_ACTION_PROMPT),
            LlmAction::Deepdive => prompt.push_str(DEEPDIVE_ACTION_PROMPT),
        }

        // 3. Issue context
        prompt.push_str(&self.build_issue_context(issue, priority_levels));

        // 4. User template (appended, not replacing)
        if let Some(template_name) = user_template {
            match self
                .template_engine
                .load_template(project_path, TemplateType::Llm, template_name)
                .await
            {
                Ok(template_content) => {
                    // Create context for template rendering
                    let context = LlmTemplateContext {
                        issue_id: issue.id.clone(),
                        display_number: issue.metadata.display_number,
                        title: issue.title.clone(),
                        description: issue.description.clone(),
                        status: issue.metadata.status.clone(),
                        priority: issue.metadata.priority,
                        priority_label: priority_label(issue.metadata.priority, priority_levels),
                        created_at: issue.metadata.created_at.clone(),
                        custom_fields: issue.metadata.custom_fields.clone(),
                        action: action.as_str().to_string(),
                        project_path: project_path.to_string_lossy().to_string(),
                    };

                    // Try to render the template with context
                    let rendered = self
                        .template_engine
                        .handlebars()
                        .render_template(&template_content, &context)
                        .unwrap_or(template_content);

                    prompt.push_str("\n---\n\n## Additional Instructions\n\n");
                    prompt.push_str(&rendered);
                }
                Err(_) => {
                    // Template not found - silently skip (user template is optional)
                    tracing::warn!("LLM template '{}' not found, skipping", template_name);
                }
            }
        }

        Ok(prompt)
    }

    /// Build the issue-specific context section
    fn build_issue_context(&self, issue: &Issue, priority_levels: u32) -> String {
        let mut context = String::new();

        context.push_str("## Issue Details\n\n");
        context.push_str(&format!(
            "**ID**: {} (#{})\n",
            issue.id, issue.metadata.display_number
        ));
        context.push_str(&format!("**Title**: {}\n", issue.title));
        context.push_str(&format!("**Status**: {}\n", issue.metadata.status));
        context.push_str(&format!(
            "**Priority**: {} ({})\n",
            issue.metadata.priority,
            priority_label(issue.metadata.priority, priority_levels)
        ));
        context.push_str(&format!("**Created**: {}\n", issue.metadata.created_at));
        context.push_str(&format!("**Updated**: {}\n\n", issue.metadata.updated_at));

        if !issue.description.is_empty() {
            context.push_str("### Description\n\n");
            context.push_str(&issue.description);
            context.push_str("\n\n");
        }

        if !issue.metadata.custom_fields.is_empty() {
            context.push_str("### Custom Fields\n\n");
            for (key, value) in &issue.metadata.custom_fields {
                context.push_str(&format!("- **{key}**: {value}\n"));
            }
            context.push('\n');
        }

        context.push_str("---\n\n");
        context
    }

    /// Get a preview of the prompt (first N characters)
    #[must_use]
    pub fn preview(prompt: &str, max_chars: usize) -> String {
        if prompt.len() > max_chars {
            format!("{}...", &prompt[..max_chars])
        } else {
            prompt.to_string()
        }
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait to access handlebars from template engine
trait TemplateEngineExt {
    fn handlebars(&self) -> &handlebars::Handlebars<'static>;
}

impl TemplateEngineExt for TemplateEngine {
    fn handlebars(&self) -> &handlebars::Handlebars<'static> {
        // Note: This requires making handlebars field accessible
        // For now, we'll use a workaround via render_template
        static HANDLEBARS: std::sync::OnceLock<handlebars::Handlebars<'static>> =
            std::sync::OnceLock::new();
        HANDLEBARS.get_or_init(handlebars::Handlebars::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::entities::issue::IssueMetadataFlat;
    use std::collections::HashMap;

    #[allow(deprecated)]
    fn create_test_issue() -> Issue {
        Issue {
            id: "test-uuid-1234".to_string(),
            issue_number: "test-uuid-1234".to_string(),
            title: "Fix authentication bug".to_string(),
            description: "Login endpoint returns 500 error when password contains special chars"
                .to_string(),
            metadata: IssueMetadataFlat {
                display_number: 42,
                status: "open".to_string(),
                priority: 1,
                created_at: "2025-01-15T10:00:00Z".to_string(),
                updated_at: "2025-01-15T10:00:00Z".to_string(),
                custom_fields: HashMap::from([
                    ("assignee".to_string(), "alice".to_string()),
                    ("component".to_string(), "auth".to_string()),
                ]),
                compacted: false,
                compacted_at: None,
                draft: false,
                deleted_at: None,
                is_org_issue: false,
                org_slug: None,
                org_display_number: None,
            },
        }
    }

    #[test]
    fn test_llm_action_as_str() {
        assert_eq!(LlmAction::Plan.as_str(), "plan");
        assert_eq!(LlmAction::Implement.as_str(), "implement");
        assert_eq!(LlmAction::Deepdive.as_str(), "deepdive");
    }

    #[test]
    fn test_llm_action_proto_conversion() {
        assert_eq!(LlmAction::from_proto(1), Some(LlmAction::Plan));
        assert_eq!(LlmAction::from_proto(2), Some(LlmAction::Implement));
        assert_eq!(LlmAction::from_proto(3), Some(LlmAction::Deepdive));
        assert_eq!(LlmAction::from_proto(0), None);
        assert_eq!(LlmAction::from_proto(99), None);

        assert_eq!(LlmAction::Plan.to_proto(), 1);
        assert_eq!(LlmAction::Implement.to_proto(), 2);
        assert_eq!(LlmAction::Deepdive.to_proto(), 3);
    }

    #[test]
    fn test_build_issue_context() {
        let builder = PromptBuilder::new();
        let issue = create_test_issue();

        let context = builder.build_issue_context(&issue, 3);

        assert!(context.contains("**ID**: test-uuid-1234 (#42)"));
        assert!(context.contains("**Title**: Fix authentication bug"));
        assert!(context.contains("**Status**: open"));
        assert!(context.contains("**Priority**: 1 (high)"));
        assert!(context.contains("Login endpoint returns 500 error"));
        assert!(context.contains("**assignee**: alice"));
        assert!(context.contains("**component**: auth"));
    }

    #[test]
    fn test_prompt_preview() {
        let prompt = "This is a test prompt that is quite long and should be truncated";

        let preview = PromptBuilder::preview(prompt, 20);
        assert_eq!(preview, "This is a test promp...");

        let short_prompt = "Short";
        let short_preview = PromptBuilder::preview(short_prompt, 20);
        assert_eq!(short_preview, "Short");
    }

    #[test]
    fn test_base_prompts_exist() {
        // Ensure base prompts are not empty
        assert!(!BASE_SYSTEM_PROMPT.is_empty());
        assert!(!PLAN_ACTION_PROMPT.is_empty());
        assert!(!IMPLEMENT_ACTION_PROMPT.is_empty());
        assert!(!DEEPDIVE_ACTION_PROMPT.is_empty());

        // Ensure they contain expected content
        assert!(BASE_SYSTEM_PROMPT.contains("Issue Context"));
        assert!(PLAN_ACTION_PROMPT.contains("Planning"));
        assert!(IMPLEMENT_ACTION_PROMPT.contains("Implementation"));
        assert!(DEEPDIVE_ACTION_PROMPT.contains("Deep Dive"));
    }
}
