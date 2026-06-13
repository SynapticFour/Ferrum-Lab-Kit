//! Core configuration, service registry, and health aggregation for **Ferrum Lab Kit**.
//! No GA4GH business logic — wiring and operational state only.

#![forbid(unsafe_code)]

mod config;
mod error;
mod health;
mod profile;
mod registry;

pub use config::{
    AuthProvider as AuthProviderKind, AuthSection, BackendSection, BeaconAccessLevel,
    BeaconServiceConfig, ConformanceSection, DrsServiceConfig, ExternalSection, FerrumSection,
    Ga4ghInfraMode, Ga4ghInfraSection, HtsgetServiceConfig, KeycloakConfig, LabKitConfig,
    LabSection, LdapAuthConfig, LsLoginConfig, MetaSection, ProfileAfricaSection,
    ProfileAuthSection, ProfileNetworkSection, ProfileResourcesSection, ProfileServicesFlags,
    ServicesSection, TesServiceConfig, TrsServiceConfig, WesServiceConfig,
};
pub use error::CoreError;
pub use health::{HealthAggregator, ServiceHealth};
pub use profile::{
    is_co_deploy, is_field_edge, load_profile_template, parse_config_or_profile, ProfileOverrides,
    ProfileTemplate,
};
pub use registry::{ServiceId, ServiceRegistry, ServiceRegistryEntry};

use std::path::Path;

/// Load `lab-kit.toml` from disk.
pub fn load_config(path: impl AsRef<Path>) -> Result<LabKitConfig, CoreError> {
    let raw = std::fs::read_to_string(path.as_ref())?;
    parse_config(&raw)
}

/// Parse TOML configuration (canonical `lab-kit.toml` or profile template).
pub fn parse_config(raw: &str) -> Result<LabKitConfig, CoreError> {
    parse_config_or_profile(raw)
}
