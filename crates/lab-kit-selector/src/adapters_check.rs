// SPDX-License-Identifier: BUSL-1.1
//! Probe configured Lab Kit backends (POSIX / SQLite / SLURM presence). These
//! adapters are Lab Kit libraries; Ferrum owns runtime GA4GH I/O.

use anyhow::Context;
use lab_kit_adapters::{MetadataStore, PosixStorageBackend, SqliteMetadataStore, StorageBackend};
use lab_kit_core::{AuthProviderKind, LabKitConfig};

pub async fn run_adapters_check(cfg: &LabKitConfig) -> anyhow::Result<()> {
    match cfg.auth.provider {
        AuthProviderKind::Ldap => {
            anyhow::bail!(
                "auth.provider=ldap is not implemented; use ls-login, keycloak, or local"
            );
        }
        AuthProviderKind::Local => {
            println!("auth: local (Ferrum offline/Passport path — Lab Kit does not bind LDAP)");
        }
        AuthProviderKind::LsLogin => {
            println!("auth: ls-login (configured; Ferrum gateway validates tokens)")
        }
        AuthProviderKind::Keycloak => {
            println!("auth: keycloak (configured; Ferrum gateway validates tokens)")
        }
        AuthProviderKind::None => println!("auth: none"),
    }

    if let Some(posix) = cfg.services.drs.as_ref().and_then(|d| d.posix.as_ref()) {
        let root = expand_home(&posix.root);
        std::fs::create_dir_all(&root).with_context(|| format!("create posix root {root}"))?;
        let store = PosixStorageBackend::new(&root);
        store
            .put_object(".lab-kit-probe", b"ok")
            .await
            .context("posix put")?;
        store
            .delete_object(".lab-kit-probe")
            .await
            .context("posix delete")?;
        println!("posix storage: ok ({root})");
    } else if let Some(backend) = cfg.backend.as_ref() {
        if backend.storage == "local-filesystem" {
            let root = expand_home(&backend.objects_path);
            std::fs::create_dir_all(&root)
                .with_context(|| format!("create objects path {root}"))?;
            let store = PosixStorageBackend::new(&root);
            store
                .put_object(".lab-kit-probe", b"ok")
                .await
                .context("posix put")?;
            store
                .delete_object(".lab-kit-probe")
                .await
                .context("posix delete")?;
            println!("posix storage: ok ({root})");
        }
    }

    if let Some(s3) = cfg.services.drs.as_ref().and_then(|d| d.s3.as_ref()) {
        println!(
            "s3: configured endpoint={} bucket={} (credentials from FERRUM_S3_* env; not probed here)",
            s3.endpoint, s3.bucket
        );
    }

    if let Some(backend) = cfg.backend.as_ref() {
        if backend.database == "sqlite" {
            let path = expand_home(&backend.sqlite_path);
            if let Some(parent) = std::path::Path::new(&path).parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let url = if path.starts_with("sqlite:") {
                path
            } else {
                format!("sqlite:{path}?mode=rwc")
            };
            let store = SqliteMetadataStore::connect(&url)
                .await
                .context("sqlite connect")?;
            store.ping().await.context("sqlite ping")?;
            println!("sqlite metadata: ok");
        }
    }

    if let Some(wes) = cfg.services.wes.as_ref() {
        if wes
            .workflow_engine
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("nextflow"))
        {
            println!("nextflow: Lab Kit does not execute pipelines — hand off to Ferrum WES");
        }
        if wes
            .compute_backend
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("slurm"))
        {
            let has_sbatch = std::process::Command::new("sbatch")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if has_sbatch {
                println!("slurm: sbatch found on PATH (not submitting a job)");
            } else if let Some(slurm) = wes.slurm.as_ref() {
                println!(
                    "slurm: no local sbatch; remote host={:?} user={:?} (not connecting)",
                    slurm.host, slurm.user
                );
            } else {
                println!("slurm: configured but sbatch not on PATH");
            }
        }
    }

    Ok(())
}

fn expand_home(p: &str) -> String {
    if let Some(rest) = p.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        return format!("{home}/{rest}");
    }
    if p == "~" {
        return std::env::var("HOME").unwrap_or_else(|_| ".".into());
    }
    p.to_string()
}
