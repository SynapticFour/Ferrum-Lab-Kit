// SPDX-License-Identifier: BUSL-1.1
//! Default Ferrum container image pins (must not float `:latest` for generated artefacts).
//!
//! Named variants (`full` / `edge` / `edge-infra`) match Ferrum GHCR tags. See
//! Ferrum `docs/IMAGE-VARIANTS.md`.

use lab_kit_core::{is_co_deploy, LabKitConfig, ServiceId, ServiceRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FerrumImageVariant {
    /// Default `ferrum-gateway` features (WES/TES/TRS included).
    Full,
    /// `--features edge` — DRS + Beacon + htsget, SQLite.
    Edge,
    /// `edge` + `external-auth` (ga4gh-infra clearinghouse + discovery).
    EdgeInfra,
}

impl FerrumImageVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Edge => "edge",
            Self::EdgeInfra => "edge-infra",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "full" => Some(Self::Full),
            "edge" => Some(Self::Edge),
            "edge-infra" => Some(Self::EdgeInfra),
            _ => None,
        }
    }

    /// Choose a published variant from the selected surfaces (not a custom compile).
    pub fn from_config(cfg: &LabKitConfig) -> Self {
        let registry = ServiceRegistry::from_config(cfg);
        if registry.is_deployed(ServiceId::Wes)
            || registry.is_deployed(ServiceId::Tes)
            || registry.is_deployed(ServiceId::Trs)
        {
            return Self::Full;
        }
        if cfg.backend.as_ref().is_some_and(|b| {
            matches!(
                b.database.to_ascii_lowercase().as_str(),
                "postgres" | "postgresql"
            )
        }) {
            return Self::Full;
        }
        if is_co_deploy(cfg) {
            return Self::EdgeInfra;
        }
        Self::Edge
    }
}

pub fn default_ferrum_image() -> &'static str {
    static PIN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    PIN.get_or_init(|| first_pin(include_str!("../../../config/ci/ferrum-image.txt")))
        .as_str()
}

pub fn default_ferrum_image_arm64() -> &'static str {
    static PIN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    PIN.get_or_init(|| first_pin(include_str!("../../../config/ci/ferrum-image-arm64.txt")))
        .as_str()
}

pub fn default_ferrum_image_edge() -> &'static str {
    static PIN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    PIN.get_or_init(|| first_pin(include_str!("../../../config/ci/ferrum-image-edge.txt")))
        .as_str()
}

pub fn default_ferrum_image_edge_infra() -> &'static str {
    static PIN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    PIN.get_or_init(|| {
        first_pin(include_str!(
            "../../../config/ci/ferrum-image-edge-infra.txt"
        ))
    })
    .as_str()
}

pub fn default_ferrum_image_for(variant: FerrumImageVariant) -> &'static str {
    match variant {
        FerrumImageVariant::Full => default_ferrum_image(),
        FerrumImageVariant::Edge => default_ferrum_image_edge(),
        FerrumImageVariant::EdgeInfra => default_ferrum_image_edge_infra(),
    }
}

pub fn pinned_ferrum_revision() -> &'static str {
    static PIN: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    PIN.get_or_init(|| first_pin(include_str!("../../../config/ci/ferrum-revision.txt")))
        .as_str()
}

fn first_pin(raw: &str) -> String {
    raw.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("ghcr.io/synapticfour/ferrum:f28f27800f1d92c6a76670c760d9beb444c368d6")
        .to_string()
}

#[cfg(test)]
mod tests {
    use lab_kit_core::parse_config;

    use super::*;

    #[test]
    fn field_edge_selects_edge_variant() {
        let cfg = parse_config(include_str!("../../../config/profiles/field-edge.toml")).unwrap();
        assert_eq!(
            FerrumImageVariant::from_config(&cfg),
            FerrumImageVariant::Edge
        );
        assert!(default_ferrum_image_for(FerrumImageVariant::Edge).ends_with("-edge"));
    }

    #[test]
    fn field_edge_infra_selects_edge_infra_variant() {
        let cfg = parse_config(include_str!(
            "../../../config/profiles/field-edge+infra.toml"
        ))
        .unwrap();
        assert_eq!(
            FerrumImageVariant::from_config(&cfg),
            FerrumImageVariant::EdgeInfra
        );
        assert!(default_ferrum_image_for(FerrumImageVariant::EdgeInfra).contains("-edge-infra"));
    }

    #[test]
    fn institute_selects_full_variant() {
        let cfg = parse_config(include_str!("../../../config/profiles/institute.toml")).unwrap();
        assert_eq!(
            FerrumImageVariant::from_config(&cfg),
            FerrumImageVariant::Full
        );
        let img = default_ferrum_image_for(FerrumImageVariant::Full);
        assert!(img.contains("ghcr.io/synapticfour/ferrum:"));
        assert!(!img.ends_with("-edge"));
        assert!(!img.contains("-edge-infra"));
    }

    #[test]
    fn beacon_only_selects_edge_variant() {
        let cfg = parse_config(include_str!("../../../config/profiles/beacon-only.toml")).unwrap();
        assert_eq!(
            FerrumImageVariant::from_config(&cfg),
            FerrumImageVariant::Edge
        );
    }
}
