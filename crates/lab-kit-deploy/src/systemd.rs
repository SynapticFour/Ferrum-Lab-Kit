// SPDX-License-Identifier: BUSL-1.1
use std::fs;
use std::path::Path;

use lab_kit_core::{is_solum_enabled, LabKitConfig, ServiceId, ServiceRegistry};

use crate::DeployError;

/// Emit a single `ferrum-gateway.service` unit (monolith) plus optional Solum unit.
///
/// Ferrum does **not** read `lab-kit.toml`. ENABLE flags go in `gateway.env`
/// (EnvironmentFile). Point `ExecStart` at the Ferrum binary and Ferrum's own config.
pub fn generate_systemd_units(cfg: &LabKitConfig, output_dir: &Path) -> Result<(), DeployError> {
    fs::create_dir_all(output_dir)?;
    let registry = ServiceRegistry::from_config(cfg);

    let enable_drs = registry.is_deployed(ServiceId::Drs);
    let enable_htsget = registry.is_deployed(ServiceId::Htsget);
    let enable_wes = registry.is_deployed(ServiceId::Wes);
    let enable_tes = registry.is_deployed(ServiceId::Tes);
    let enable_beacon = registry.is_deployed(ServiceId::Beacon);
    let enable_trs = registry.is_deployed(ServiceId::Trs);
    let any =
        enable_drs || enable_htsget || enable_wes || enable_tes || enable_beacon || enable_trs;

    if any {
        let env = format!(
            "FERRUM_BIND=0.0.0.0:8080\n\
             FERRUM_SERVICES__ENABLE_DRS={enable_drs}\n\
             FERRUM_SERVICES__ENABLE_HTSGET={enable_htsget}\n\
             FERRUM_SERVICES__ENABLE_WES={enable_wes}\n\
             FERRUM_SERVICES__ENABLE_TES={enable_tes}\n\
             FERRUM_SERVICES__ENABLE_BEACON={enable_beacon}\n\
             FERRUM_SERVICES__ENABLE_TRS={enable_trs}\n"
        );
        fs::write(output_dir.join("gateway.env"), env.as_bytes())?;

        let unit = r#"[Unit]
Description=Ferrum Lab Kit — ferrum-gateway (monolith)
Documentation=https://github.com/SynapticFour/Ferrum-Lab-Kit
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=-/etc/ferrum/gateway.env
# Ferrum reads Ferrum config — not lab-kit.toml. See Ferrum docs/INSTALLATION.md.
ExecStart=/usr/local/bin/ferrum
Restart=on-failure

[Install]
WantedBy=multi-user.target
"#;
        fs::write(output_dir.join("ferrum-gateway.service"), unit.as_bytes())?;
    }

    if is_solum_enabled(cfg) {
        let solum_unit = r#"[Unit]
Description=Ferrum Lab Kit — Solum sidecar (consent companion)
Documentation=https://github.com/SynapticFour/Solum
After=network-online.target
Wants=network-online.target
Before=ferrum-gateway.service

[Service]
Type=simple
Environment=SOLUM_SIDECAR_BIND=0.0.0.0:8787
EnvironmentFile=-/etc/solum/sidecar.env
ExecStart=/usr/local/bin/solum-sidecar --bind 0.0.0.0:8787
Restart=on-failure

[Install]
WantedBy=multi-user.target
"#;
        fs::write(
            output_dir.join("solum-sidecar.service"),
            solum_unit.as_bytes(),
        )?;
    }

    Ok(())
}
