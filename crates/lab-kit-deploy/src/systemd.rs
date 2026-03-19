use std::fs;
use std::path::Path;

use lab_kit_core::{LabKitConfig, ServiceId, ServiceRegistry};

use crate::DeployError;

/// Emit `ferrum-<service>.service` unit stubs for login-node deployments.
pub fn generate_systemd_units(cfg: &LabKitConfig, output_dir: &Path) -> Result<(), DeployError> {
    fs::create_dir_all(output_dir)?;
    let registry = ServiceRegistry::from_config(cfg);
    for e in &registry.entries {
        if !e.deploy {
            continue;
        }
        let name = match e.id {
            ServiceId::Drs => "drs",
            ServiceId::Htsget => "htsget",
            ServiceId::Wes => "wes",
            ServiceId::Tes => "tes",
            ServiceId::Beacon => "beacon",
            ServiceId::Trs => "trs",
            ServiceId::Auth => "auth",
        };
        let unit = format!(
            r#"[Unit]
Description=Ferrum Lab Kit — {name} (via Ferrum binaries)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
# Replace ExecStart with Ferrum release binary path + config from lab-kit.toml
ExecStart=/usr/local/bin/ferrum-{name} --config /etc/ferrum/lab-kit.toml
Restart=on-failure

[Install]
WantedBy=multi-user.target
"#
        );
        fs::write(
            output_dir.join(format!("ferrum-{name}.service")),
            unit.as_bytes(),
        )?;
    }
    Ok(())
}
