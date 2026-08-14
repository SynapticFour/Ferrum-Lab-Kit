//! SLURM client on a **remote** login node via the system `ssh` binary (`sbatch`, `squeue`).
//!
//! # Auth
//! Use non-interactive authentication: SSH keys, `ssh-agent`, or host-based auth.
//! `BatchMode=yes` is set so password prompts fail fast instead of hanging.

use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::compute::{ComputeBackend, ComputeError, ComputeJobSpec, ComputeJobStatus};
use crate::slurm::{parse_sbatch_job_id, sbatch_flag_args};

/// Compute backend: run `sbatch` / `squeue` on a cluster login node over SSH.
#[derive(Debug, Clone)]
pub struct SlurmSshComputeBackend {
    /// SSH destination: `user@host` or `host` (then `~/.ssh/config` supplies user).
    pub ssh_target: String,
    /// Optional SSH port (`ssh -p`).
    pub ssh_port: Option<u16>,
    /// Optional identity file (`ssh -i`).
    pub identity_file: Option<PathBuf>,
    /// Extra `ssh` arguments **before** the remote destination (e.g. `-o`, `ProxyJump=jumphost`).
    ///
    /// Operator-supplied only — never interpolate untrusted input; these are raw ssh argv.
    pub extra_ssh_args: Vec<String>,
    /// Optional `UserKnownHostsFile`. When unset, OpenSSH uses `~/.ssh/known_hosts`.
    /// `StrictHostKeyChecking=yes` requires the host to already be listed there.
    pub known_hosts: Option<PathBuf>,
    /// Optional SLURM partition (`sbatch -p`).
    pub partition: Option<String>,
}

impl SlurmSshComputeBackend {
    pub fn new(ssh_target: impl Into<String>) -> Self {
        Self {
            ssh_target: ssh_target.into(),
            ssh_port: None,
            identity_file: None,
            extra_ssh_args: Vec::new(),
            known_hosts: None,
            partition: None,
        }
    }

    fn base_ssh(&self) -> Command {
        let mut cmd = Command::new("ssh");
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        cmd.arg("-o").arg("BatchMode=yes");
        cmd.arg("-o").arg("StrictHostKeyChecking=yes");
        if let Some(ref kh) = self.known_hosts {
            cmd.arg("-o")
                .arg(format!("UserKnownHostsFile={}", kh.display()));
        }
        if let Some(p) = self.ssh_port {
            cmd.args(["-p", &p.to_string()]);
        }
        if let Some(ref path) = self.identity_file {
            let s = path.to_str().unwrap_or("");
            if !s.is_empty() {
                cmd.args(["-i", s]);
            }
        }
        for a in &self.extra_ssh_args {
            cmd.arg(a);
        }
        cmd.arg(&self.ssh_target);
        cmd
    }
}

#[async_trait]
impl ComputeBackend for SlurmSshComputeBackend {
    async fn submit(&self, spec: ComputeJobSpec) -> Result<String, ComputeError> {
        let mut cmd = self.base_ssh();
        cmd.stdin(Stdio::piped());
        cmd.arg("sbatch");
        for a in sbatch_flag_args(&spec, self.partition.as_deref()) {
            cmd.arg(a);
        }
        cmd.arg("/dev/stdin");

        let mut child = cmd.spawn().map_err(|e| {
            ComputeError::Scheduler(format!(
                "failed to run ssh sbatch (is `ssh` installed and reachable?): {e}"
            ))
        })?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(format!("#!/bin/bash\n{}\n", spec.script).as_bytes())
                .await?;
        }
        let out = child.wait_with_output().await?;
        if !out.status.success() {
            return Err(ComputeError::Scheduler(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
        Ok(parse_sbatch_job_id(&String::from_utf8_lossy(&out.stdout)))
    }

    async fn status(&self, job_id: &str) -> Result<ComputeJobStatus, ComputeError> {
        let mut cmd = self.base_ssh();
        cmd.args(["squeue", "-h", "-j", job_id, "-o", "%T"]);

        let out = cmd
            .output()
            .await
            .map_err(|e| ComputeError::Scheduler(format!("failed to run ssh squeue: {e}")))?;
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
