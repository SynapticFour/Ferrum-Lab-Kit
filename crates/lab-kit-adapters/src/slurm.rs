use std::process::Stdio;

use async_trait::async_trait;
use tokio::process::Command;

use crate::compute::{ComputeBackend, ComputeError, ComputeJobSpec, ComputeJobStatus};

/// SLURM via local `sbatch`/`squeue` (login node deployment).
#[derive(Default)]
pub struct SlurmComputeBackend {
    pub partition: Option<String>,
}

pub(crate) fn sanitize_job_name(name: &str) -> String {
    let s: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .take(64)
        .collect();
    if s.is_empty() {
        "lab-kit".into()
    } else {
        s
    }
}

pub(crate) fn parse_sbatch_job_id(stdout: &str) -> String {
    stdout
        .split_whitespace()
        .last()
        .unwrap_or("unknown")
        .trim()
        .to_string()
}

pub(crate) fn sbatch_flag_args(spec: &ComputeJobSpec, partition: Option<&str>) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(p) = partition {
        args.extend(["-p".into(), p.to_string()]);
    }
    if let Some(c) = spec.cpus {
        args.extend(["-c".into(), c.to_string()]);
    }
    if let Some(m) = spec.memory_mb {
        args.extend(["--mem".into(), format!("{m}M")]);
    }
    args.extend(["--job-name".into(), sanitize_job_name(&spec.name)]);
    args
}

#[async_trait]
impl ComputeBackend for SlurmComputeBackend {
    async fn submit(&self, spec: ComputeJobSpec) -> Result<String, ComputeError> {
        let tmp = tempfile::Builder::new()
            .prefix("lab-kit-sbatch-")
            .suffix(".sh")
            .tempfile()
            .map_err(ComputeError::Io)?;
        std::fs::write(tmp.path(), format!("#!/bin/bash\n{}\n", spec.script))?;

        let mut cmd = Command::new("sbatch");
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for a in sbatch_flag_args(&spec, self.partition.as_deref()) {
            cmd.arg(a);
        }
        cmd.arg(tmp.path());
        let child = cmd.spawn().map_err(|e| {
            ComputeError::Scheduler(format!(
                "failed to spawn sbatch (is SLURM client installed?): {e}"
            ))
        })?;
        let out = child.wait_with_output().await?;
        drop(tmp);
        if !out.status.success() {
            return Err(ComputeError::Scheduler(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
        Ok(parse_sbatch_job_id(&String::from_utf8_lossy(&out.stdout)))
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
