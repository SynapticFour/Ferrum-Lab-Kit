use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("engine: {0}")]
    Engine(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunSpec {
    pub pipeline: String,
    #[serde(default)]
    pub profile: Option<String>,
}

#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    async fn run(&self, spec: WorkflowRunSpec) -> Result<String, WorkflowError>;
}

/// Nextflow integration hook — actual execution is delegated to Ferrum WES/TES.
pub struct NextflowWorkflowEngine;

#[async_trait]
impl WorkflowEngine for NextflowWorkflowEngine {
    async fn run(&self, spec: WorkflowRunSpec) -> Result<String, WorkflowError> {
        Err(WorkflowError::Engine(format!(
            "Nextflow dispatch not executed in Lab Kit — hand off to Ferrum WES for {}",
            spec.pipeline
        )))
    }
}
