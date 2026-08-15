//! Clone pinned Ferrum (optional) and `docker build` a named gateway variant.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::images::{pinned_ferrum_revision, FerrumImageVariant};
use crate::DeployError;

const FERRUM_REMOTE_DEFAULT: &str = "https://github.com/SynapticFour/Ferrum.git";

#[derive(Debug, Clone)]
pub struct BuildImageOptions {
    pub variant: FerrumImageVariant,
    /// Extra cargo features; overrides the named variant when set (Ferrum `FERRUM_GATEWAY_FEATURES`).
    pub features: Option<String>,
    pub platform: Option<String>,
    pub ferrum_src: Option<PathBuf>,
    pub tag: String,
    pub dry_run: bool,
}

impl BuildImageOptions {
    pub fn tag_for(variant: FerrumImageVariant) -> String {
        format!("ferrum:lab-kit-{}", variant.as_str())
    }
}

/// Build (or print) a Ferrum gateway image. Returns the image tag.
pub fn build_ferrum_image(opts: &BuildImageOptions) -> Result<String, DeployError> {
    let src = resolve_ferrum_src(opts)?;
    let dockerfile = src.join("deploy/Dockerfile");
    if !opts.dry_run && !dockerfile.is_file() {
        return Err(DeployError::Msg(format!(
            "Ferrum Dockerfile not found at {} (need a Ferrum checkout at the Lab Kit pin)",
            dockerfile.display()
        )));
    }

    let mut args: Vec<String> = vec![
        "build".into(),
        "-f".into(),
        dockerfile.to_string_lossy().into_owned(),
        "--build-arg".into(),
        format!("FERRUM_VARIANT={}", opts.variant.as_str()),
        "--build-arg".into(),
        format!(
            "FERRUM_GATEWAY_FEATURES={}",
            opts.features.as_deref().unwrap_or("")
        ),
        "--build-arg".into(),
        format!("FERRUM_GIT_SHA={}", pinned_ferrum_revision()),
        "--build-arg".into(),
        format!("FERRUM_BUILD_PROFILE={}", opts.variant.as_str()),
        "-t".into(),
        opts.tag.clone(),
    ];
    if let Some(p) = &opts.platform {
        args.push("--platform".into());
        args.push(p.clone());
    }
    args.push(src.to_string_lossy().into_owned());

    if opts.dry_run {
        tracing::info!(
            docker = %format!("docker {}", args.join(" ")),
            "dry-run: not invoking docker"
        );
        return Ok(opts.tag.clone());
    }

    let status = Command::new("docker")
        .args(&args)
        .status()
        .map_err(|e| DeployError::Msg(format!("docker build failed to start: {e}")))?;
    if !status.success() {
        return Err(DeployError::Msg(format!(
            "docker build exited {}",
            status.code().unwrap_or(-1)
        )));
    }
    Ok(opts.tag.clone())
}

fn resolve_ferrum_src(opts: &BuildImageOptions) -> Result<PathBuf, DeployError> {
    if let Some(p) = &opts.ferrum_src {
        return Ok(p.clone());
    }
    if let Ok(p) = std::env::var("FERRUM_SRC") {
        let path = PathBuf::from(p);
        if path.is_dir() {
            return Ok(path);
        }
    }
    if opts.dry_run {
        return Ok(PathBuf::from("/tmp/ferrum-src-dry-run"));
    }
    clone_pinned_ferrum()
}

fn clone_pinned_ferrum() -> Result<PathBuf, DeployError> {
    let sha = pinned_ferrum_revision();
    let cache = cache_dir()?.join("ferrum").join(sha);
    if cache.join("deploy/Dockerfile").is_file() {
        return Ok(cache);
    }
    std::fs::create_dir_all(&cache)?;
    let remote =
        std::env::var("FERRUM_REMOTE").unwrap_or_else(|_| FERRUM_REMOTE_DEFAULT.to_string());
    git(&["init"], &cache)?;
    let _ = git(&["remote", "remove", "origin"], &cache);
    git(&["remote", "add", "origin", &remote], &cache)?;
    git(&["fetch", "--depth", "1", "origin", sha], &cache)?;
    git(&["checkout", "--force", "FETCH_HEAD"], &cache)?;
    Ok(cache)
}

fn cache_dir() -> Result<PathBuf, DeployError> {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(xdg).join("ferrum-lab-kit"));
    }
    let home = std::env::var("HOME").map_err(|_| DeployError::Msg("HOME is unset".into()))?;
    Ok(PathBuf::from(home).join(".cache/ferrum-lab-kit"))
}

fn git(args: &[&str], cwd: &Path) -> Result<(), DeployError> {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|e| DeployError::Msg(format!("git {:?} failed to start: {e}", args)))?;
    if !status.success() {
        return Err(DeployError::Msg(format!(
            "git {:?} exited {}",
            args,
            status.code().unwrap_or(-1)
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_does_not_need_docker() {
        let tag = build_ferrum_image(&BuildImageOptions {
            variant: FerrumImageVariant::Edge,
            features: None,
            platform: Some("linux/arm64".into()),
            ferrum_src: None,
            tag: BuildImageOptions::tag_for(FerrumImageVariant::Edge),
            dry_run: true,
        })
        .unwrap();
        assert_eq!(tag, "ferrum:lab-kit-edge");
    }
}
