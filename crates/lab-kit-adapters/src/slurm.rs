use std::process::Stdio;

use async_trait::async_trait;
use tokio::process::Command;

use crate::compute::{ComputeBackend, ComputeError, ComputeJobSpec, ComputeJobStatus};

/// SLURM via local `sbatch`/`squeue` (login node deployment).
#[derive(Default)]
pub struct SlurmComputeBackend {
    pub partition: Option<String>,
}

#[async_trait]
impl ComputeBackend for SlurmComputeBackend {
    async fn submit(&self, spec: ComputeJobSpec) -> Result<String, ComputeError> {
        let mut cmd = Command::new("sbatch");
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(p) = &self.partition {
            cmd.args(["-p", p]);
        }
        cmd.arg("--job-name").arg(&spec.name);
        cmd.arg("--wrap").arg(&spec.script);
        let child = cmd.spawn().map_err(|e| {
            ComputeError::Scheduler(format!(
                "failed to spawn sbatch (is SLURM client installed?): {e}"
            ))
        })?;
        let out = child.wait_with_output().await?;
        if !out.status.success() {
            return Err(ComputeError::Scheduler(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        // Submitted batch job 12345
        let id = stdout
            .split_whitespace()
            .last()
            .unwrap_or("unknown")
            .trim()
            .to_string();
        Ok(id)
    }

    async fn status(&self, job_id: &str) -> Result<ComputeJobStatus, ComputeError> {
        let out = Command::new("squeue")
            .args(["-h", "-j", job_id, "-o", "%T"])
            .output()
            .await?;
        let state = if out.status.success() {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        } else {
            "UNKNOWN".into()
        };
        Ok(ComputeJobStatus {
            job_id: job_id.to_string(),
            state,
        })
    }
}
