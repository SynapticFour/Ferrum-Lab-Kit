//! Generate deployment artifacts from [`lab_kit_core::LabKitConfig`] and [`lab_kit_core::ServiceRegistry`].

#![forbid(unsafe_code)]

mod compose;
mod error;
mod helm;
mod routing;
mod systemd;

pub use compose::generate_compose_file;
pub use error::DeployError;
pub use helm::generate_helm_values;
pub use routing::write_external_upstreams_next_to_compose;
pub use systemd::generate_systemd_units;
