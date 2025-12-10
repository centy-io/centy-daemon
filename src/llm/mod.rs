pub mod agent;
pub mod config;
pub mod prompt;
pub mod work;

#[allow(unused_imports)]
pub use agent::{spawn_agent, AgentError};
#[allow(unused_imports)]
pub use config::{
    get_effective_local_config, has_global_config, has_project_config, write_global_local_config,
    write_project_local_config, AgentConfig, AgentType, LocalConfigError, LocalLlmConfig,
};
#[allow(unused_imports)]
pub use prompt::{
    LlmAction, PromptBuilder, PromptError,
};
#[allow(unused_imports)]
pub use work::{
    clear_work_session, is_process_running, read_work_session,
    record_work_session, LlmWorkSession, WorkTrackingError,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Config error: {0}")]
    ConfigError(#[from] LocalConfigError),

    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),

    #[error("Prompt error: {0}")]
    PromptError(#[from] PromptError),

    #[error("Work tracking error: {0}")]
    WorkTrackingError(#[from] WorkTrackingError),

    #[error("Issue error: {0}")]
    IssueError(#[from] crate::issue::IssueCrudError),
}
