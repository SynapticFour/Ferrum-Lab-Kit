use std::fs;
use std::path::Path;

use lab_kit_core::{LabKitConfig, ServiceId, ServiceRegistry};
use serde::Serialize;

use crate::DeployError;

/// Write Helm-style `values` with enabled flags, images, and optional `externalUrl` per service.
pub fn generate_helm_values(cfg: &LabKitConfig, output_path: &Path) -> Result<(), DeployError> {
    let registry = ServiceRegistry::from_config(cfg);
    let mut drs = svc_defaults(ServiceId::Drs);
    let mut htsget = svc_defaults(ServiceId::Htsget);
    let mut wes = svc_defaults(ServiceId::Wes);
    let mut tes = svc_defaults(ServiceId::Tes);
    let mut beacon = svc_defaults(ServiceId::Beacon);
    let mut trs = svc_defaults(ServiceId::Trs);
    let mut auth = svc_defaults(ServiceId::Auth);

    for e in &registry.entries {
        let target = match e.id {
            ServiceId::Drs => &mut drs,
            ServiceId::Htsget => &mut htsget,
            ServiceId::Wes => &mut wes,
            ServiceId::Tes => &mut tes,
            ServiceId::Beacon => &mut beacon,
            ServiceId::Trs => &mut trs,
            ServiceId::Auth => &mut auth,
        };
        target.enabled = e.deploy;
        if let Some(u) = &e.external_base {
            target.external_url = Some(u.to_string());
        }
    }

    let root = HelmRoot {
        global: GlobalVals {
            image_registry: String::new(),
        },
        lab: HelmLab {
            name: cfg.lab.name.clone(),
            environment: cfg.lab.environment.clone(),
        },
        services: ServicesVals {
            drs,
            htsget,
            wes,
            tes,
            beacon,
            trs,
            auth,
        },
        auth: AuthVals {
            ls_login: LsLoginVals {
                issuer: cfg
                    .auth
                    .ls_login
                    .as_ref()
                    .map(|l| l.issuer.clone())
                    .unwrap_or_else(|| "https://login.elixir-czech.org/oidc/".to_string()),
            },
        },
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_path, serde_yaml::to_string(&root)?)?;
    Ok(())
}

fn svc_defaults(id: ServiceId) -> ServiceHelmVals {
    ServiceHelmVals {
        enabled: false,
        image: default_image(id),
        external_url: None,
    }
}

fn default_image(id: ServiceId) -> String {
    let name = match id {
        ServiceId::Drs => "ferrum-drs",
        ServiceId::Htsget => "ferrum-htsget",
        ServiceId::Wes => "ferrum-wes",
        ServiceId::Tes => "ferrum-tes",
        ServiceId::Beacon => "ferrum-beacon",
        ServiceId::Trs => "ferrum-trs",
        ServiceId::Auth => "ferrum-auth-proxy",
    };
    format!("synapticfour/{name}:latest")
}

#[derive(Serialize)]
struct HelmRoot {
    global: GlobalVals,
    lab: HelmLab,
    services: ServicesVals,
    auth: AuthVals,
}

#[derive(Serialize)]
struct GlobalVals {
    #[serde(rename = "imageRegistry")]
    image_registry: String,
}

#[derive(Serialize)]
struct HelmLab {
    name: String,
    environment: String,
}

#[derive(Serialize)]
struct ServicesVals {
    drs: ServiceHelmVals,
    htsget: ServiceHelmVals,
    wes: ServiceHelmVals,
    tes: ServiceHelmVals,
    beacon: ServiceHelmVals,
    trs: ServiceHelmVals,
    auth: ServiceHelmVals,
}

#[derive(Serialize)]
struct ServiceHelmVals {
    enabled: bool,
    image: String,
    #[serde(rename = "externalUrl", skip_serializing_if = "Option::is_none")]
    external_url: Option<String>,
}

#[derive(Serialize)]
struct AuthVals {
    #[serde(rename = "lsLogin")]
    ls_login: LsLoginVals,
}

#[derive(Serialize)]
struct LsLoginVals {
    issuer: String,
}
