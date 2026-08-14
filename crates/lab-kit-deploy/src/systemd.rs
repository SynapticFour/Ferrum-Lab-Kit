use std::fs;
use std::path::Path;

use lab_kit_core::{is_solum_enabled, LabKitConfig, ServiceId, ServiceRegistry};

use crate::DeployError;

/// Emit a single `ferrum-gateway.service` unit (monolith) plus optional Solum unit.
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
        let unit = format!(
            r#"[Unit]
Description=Ferrum Lab Kit — ferrum-gateway (monolith)
Documentation=https://github.com/SynapticFour/Ferrum-Lab-Kit
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
Environment=FERRUM_BIND=0.0.0.0:8080
Environment=FERRUM_SERVICES__ENABLE_DRS={enable_drs}
Environment=FERRUM_SERVICES__ENABLE_HTSGET={enable_htsget}
Environment=FERRUM_SERVICES__ENABLE_WES={enable_wes}
Environment=FERRUM_SERVICES__ENABLE_TES={enable_tes}
Environment=FERRUM_SERVICES__ENABLE_BEACON={enable_beacon}
Environment=FERRUM_SERVICES__ENABLE_TRS={enable_trs}
# Replace ExecStart with your Ferrum release binary path
ExecStart=/usr/local/bin/ferrum --config /etc/ferrum/lab-kit.toml
Restart=on-failure

[Install]
WantedBy=multi-user.target
"#
        );
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
Environment=SOLUM_PROFILE=/etc/solum/dev-local.toml
# Set SOLUM_SIDECAR_TOKEN and match FERRUM_SOLUM__* on ferrum-gateway
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
