mod action_builders;
mod action_builders_extra;
mod actions;
mod config_to_proto;
mod convert_entity;
mod convert_infra;
mod convert_link;
pub mod error_mapping;
pub mod handlers;
mod helpers;
mod hooks_helper;
mod proto_to_config;
mod resolve;
mod startup;
pub mod structured_error;
mod trait_impl;
mod validate_config;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::watch;

// Import generated protobuf types
pub mod proto {
    #![allow(clippy::pedantic)]
    #![allow(clippy::all)]
    tonic::include_proto!("centy.v1");
}

/// Signal type for daemon shutdown/restart
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShutdownSignal {
    None,
    Shutdown,
    Restart,
}

pub struct CentyDaemonService {
    shutdown_tx: Arc<watch::Sender<ShutdownSignal>>,
    exe_path: Option<PathBuf>,
}

impl CentyDaemonService {
    #[must_use]
    pub fn new(shutdown_tx: Arc<watch::Sender<ShutdownSignal>>, exe_path: Option<PathBuf>) -> Self {
        // Spawn background task to infer organizations for ungrouped projects on startup
        tokio::spawn(async {
            startup::startup_org_inference().await;
        });

        Self {
            shutdown_tx,
            exe_path,
        }
    }
}
