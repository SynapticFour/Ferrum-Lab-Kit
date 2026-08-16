// SPDX-License-Identifier: BUSL-1.1
//! Generate deployment artifacts from [`lab_kit_core::LabKitConfig`] and [`lab_kit_core::ServiceRegistry`].

#![forbid(unsafe_code)]

mod build_image;
mod compose;
mod error;
mod helm;
mod images;
mod infra_secrets;
mod pi_bundle;
mod routing;
mod systemd;

pub use build_image::{build_ferrum_image, BuildImageOptions};
pub use compose::{
    generate_compose_file, render_compose_yaml, write_compose_sidecars, ComposeOptions,
};
pub use error::DeployError;
pub use helm::generate_helm_values;
pub use images::{
    default_ferrum_image, default_ferrum_image_arm64, default_ferrum_image_edge,
    default_ferrum_image_edge_infra, default_ferrum_image_for, pinned_ferrum_revision,
    FerrumImageVariant,
};
pub use infra_secrets::generate_infra_secrets;
pub use pi_bundle::{generate_raspberry_pi_bundle, RaspberryPiBundleOptions};
pub use routing::write_external_upstreams_next_to_compose;
pub use systemd::generate_systemd_units;
