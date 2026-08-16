// SPDX-License-Identifier: BUSL-1.1
//! Compile-time link to **[Ferrum](https://github.com/SynapticFour/Ferrum)** (`ferrum-core`).
//! Lab Kit does not re-export the full platform; integrators use this crate to share types
//! (config, errors, auth) with Ferrum gateways and services.

#![forbid(unsafe_code)]

pub use ferrum_core;

/// Git revision pinned in `Cargo.toml` (mirror `config/ci/ferrum-revision.txt`).
pub fn ferrum_git_rev() -> &'static str {
    static REV: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    REV.get_or_init(|| {
        include_str!("../../../config/ci/ferrum-revision.txt")
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && !l.starts_with('#'))
            .unwrap_or("unknown")
            .to_string()
    })
    .as_str()
}

/// Upstream repository URL.
pub const FERRUM_GIT_URL: &str = "https://github.com/SynapticFour/Ferrum.git";

/// Smoketest that `ferrum-core` symbols resolve (used by `lab-kit ferrum check`).
pub fn ferrum_core_type_name() -> &'static str {
    std::any::type_name::<ferrum_core::FerrumError>()
}

/// Exercise `ferrum-core` types that Lab Kit depends on staying ABI-stable.
pub fn ferrum_core_link_check() -> Result<String, String> {
    let err = ferrum_core::FerrumError::ValidationError("lab-kit-link".into());
    let msg = err.to_string();
    if !msg.contains("lab-kit-link") {
        return Err(format!("unexpected FerrumError Display: {msg}"));
    }
    let _policy = std::any::type_name::<ferrum_core::SsrfPolicy>();
    if _policy.is_empty() {
        return Err("SsrfPolicy type name empty".into());
    }
    Ok(format!(
        "ferrum-core ok: {ty} @ {rev}\n{url}",
        ty = ferrum_core_type_name(),
        rev = ferrum_git_rev(),
        url = FERRUM_GIT_URL
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ferrum_core_link_resolves() {
        ferrum_core_link_check().expect("ferrum-core link");
        assert_eq!(ferrum_git_rev().len(), 40);
    }
}
