//! Compile-time link to **[Ferrum](https://github.com/SynapticFour/Ferrum)** (`ferrum-core`).
//! Lab Kit does not re-export the full platform; integrators use this crate to share types
//! (config, errors, auth) with Ferrum gateways and services.

#![forbid(unsafe_code)]

pub use ferrum_core;

/// Git revision pinned in `Cargo.toml` (mirror `config/ci/ferrum-revision.txt`).
pub const FERRUM_GIT_REV: &str = "9eb113bcab8f23e0c34b63baf3d19ecd8f6f480a";

/// Upstream repository URL.
pub const FERRUM_GIT_URL: &str = "https://github.com/SynapticFour/Ferrum.git";

/// Smoketest that `ferrum-core` symbols resolve (used by `lab-kit ferrum check`).
pub fn ferrum_core_type_name() -> &'static str {
    std::any::type_name::<ferrum_core::FerrumError>()
}
