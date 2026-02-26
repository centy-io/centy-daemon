use crate::manifest::CentyManifest;
use std::collections::HashSet;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Manifest error: {0}")]
    ManifestError(#[from] crate::manifest::ManifestError),
    #[error("Plan error: {0}")]
    PlanError(#[from] super::super::plan::PlanError),
    #[error("Config error: {0}")]
    ConfigError(#[from] mdstore::ConfigError),
}
/// User decisions for reconciliation
#[derive(Debug, Clone, Default)]
pub struct ReconciliationDecisions {
    /// Paths of files to restore
    pub restore: HashSet<String>,
    /// Paths of files to reset
    pub reset: HashSet<String>,
}
/// Result of reconciliation execution
#[derive(Debug, Clone, Default)]
pub struct ReconciliationResult {
    pub created: Vec<String>,
    pub restored: Vec<String>,
    pub reset: Vec<String>,
    pub skipped: Vec<String>,
    pub manifest: CentyManifest,
}
