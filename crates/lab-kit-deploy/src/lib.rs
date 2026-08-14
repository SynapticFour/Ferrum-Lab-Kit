//! Generate deployment artifacts from [`lab_kit_core::LabKitConfig`] and [`lab_kit_core::ServiceRegistry`].

#![forbid(unsafe_code)]

mod compose;
mod error;
mod helm;
mod pi_bundle;
mod routing;
mod systemd;

pub use compose::{
    generate_compose_file, render_compose_yaml, write_compose_sidecars, ComposeOptions,
};
pub use error::DeployError;
pub use helm::generate_helm_values;
pub use pi_bundle::{generate_raspberry_pi_bundle, RaspberryPiBundleOptions};
pub use routing::write_external_upstreams_next_to_compose;
pub use systemd::generate_systemd_units;
