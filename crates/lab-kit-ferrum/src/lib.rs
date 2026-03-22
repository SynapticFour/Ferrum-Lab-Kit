//! Compile-time link to **[Ferrum](https://github.com/SynapticFour/Ferrum)** (`ferrum-core`).
//! Lab Kit does not re-export the full platform; integrators use this crate to share types
//! (config, errors, auth) with Ferrum gateways and services.

#![forbid(unsafe_code)]

pub use ferrum_core;

/// Git revision pinned in `Cargo.toml` (mirror `config/ci/ferrum-revision.txt`).
pub const FERRUM_GIT_REV: &str = "6bee3495b8347d5ef914fdae97fc7214070604f3";

/// Upstream repository URL.
pub const FERRUM_GIT_URL: &str = "https://github.com/SynapticFour/Ferrum.git";

/// Smoketest that `ferrum-core` symbols resolve (used by `lab-kit ferrum check`).
pub fn ferrum_core_type_name() -> &'static str {
    std::any::type_name::<ferrum_core::FerrumError>()
}
