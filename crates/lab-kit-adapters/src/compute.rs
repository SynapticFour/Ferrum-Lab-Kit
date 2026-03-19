use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComputeError {
    #[error("scheduler: {0}")]
    Scheduler(String),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJobSpec {
    pub name: String,
    pub script: String,
    #[serde(default)]
    pub cpus: Option<u32>,
    #[serde(default)]
    pub memory_mb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJobStatus {
    pub job_id: String,
    pub state: String,
}

#[async_trait]
pub trait ComputeBackend: Send + Sync {
    async fn submit(&self, spec: ComputeJobSpec) -> Result<String, ComputeError>;
    async fn status(&self, job_id: &str) -> Result<ComputeJobStatus, ComputeError>;
}
