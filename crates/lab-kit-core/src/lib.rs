//! Core configuration, service registry, and health aggregation for **Ferrum Lab Kit**.
//! No GA4GH business logic — wiring and operational state only.

#![forbid(unsafe_code)]

mod config;
mod error;
mod health;
mod registry;

pub use config::{
    AuthProvider as AuthProviderKind, AuthSection, BeaconAccessLevel, BeaconServiceConfig,
    DrsServiceConfig, ExternalSection, FerrumSection, HtsgetServiceConfig, KeycloakConfig,
    LabKitConfig, LabSection, LdapAuthConfig, LsLoginConfig, ServicesSection, TesServiceConfig,
    TrsServiceConfig, WesServiceConfig,
};
pub use error::CoreError;
pub use health::{HealthAggregator, ServiceHealth};
pub use registry::{ServiceId, ServiceRegistry, ServiceRegistryEntry};

use std::path::Path;

/// Load `lab-kit.toml` from disk.
pub fn load_config(path: impl AsRef<Path>) -> Result<LabKitConfig, CoreError> {
    let raw = std::fs::read_to_string(path.as_ref())?;
    parse_config(&raw)
}

/// Parse TOML configuration.
pub fn parse_config(raw: &str) -> Result<LabKitConfig, CoreError> {
    let cfg: LabKitConfig = toml::from_str(raw)?;
    cfg.validate()?;
    Ok(cfg)
}
