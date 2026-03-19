use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use lab_kit_auth::LsLoginOidc;
use lab_kit_core::{
    load_config, AuthProviderKind, AuthSection, BeaconAccessLevel, BeaconServiceConfig,
    LabKitConfig, LabSection, LsLoginConfig, ServiceRegistry, ServicesSection,
};
use lab_kit_deploy::{generate_compose_file, generate_helm_values, generate_systemd_units};
use lab_kit_report::generate_reports;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "lab-kit")]
#[command(about = "Ferrum Lab Kit — selective GA4GH deployment & integration")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Interactive wizard — writes `lab-kit.toml`.
    Init {
        #[arg(short, long, default_value = "lab-kit.toml")]
        output: PathBuf,
    },
    Generate {
        #[command(subcommand)]
        target: GenerateTarget,
    },
    /// Poll configured health endpoints.
    Status {
        #[arg(short, long, default_value = "lab-kit.toml")]
        config: PathBuf,
    },
    Conformance {
        #[command(subcommand)]
        action: ConformanceAction,
    },
    /// Verify compile-time link to SynapticFour/Ferrum (`ferrum-core`).
    Ferrum {
        #[command(subcommand)]
        action: FerrumAction,
    },
}

#[derive(Subcommand)]
enum FerrumAction {
    /// Confirm `ferrum-core` resolves (git pin in `lab-kit-ferrum`).
    Check,
}

#[derive(Subcommand)]
enum GenerateTarget {
    Compose {
        #[arg(short, long, default_value = "lab-kit.toml")]
        config: PathBuf,
        #[arg(long, default_value = "deploy/docker-compose")]
        fragments: PathBuf,
        #[arg(short, long, default_value = "docker-compose.yml")]
        output: PathBuf,
        /// Write merged compose to stdout instead of a file.
        #[arg(long, default_value_t = false)]
        stdout: bool,
    },
    Helm {
        #[arg(short, long, default_value = "lab-kit.toml")]
        config: PathBuf,
        #[arg(long, default_value = "generated/helm-values.yaml")]
        output: PathBuf,
        #[arg(long, default_value_t = false)]
        stdout: bool,
    },
    Systemd {
        #[arg(short, long, default_value = "lab-kit.toml")]
        config: PathBuf,
        #[arg(long, default_value = "generated/systemd")]
        output_dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum ConformanceAction {
    /// Run HelixTest (expects `helixtest` on PATH or set HELIXTEST_BIN).
    Run {
        #[arg(short, long, default_value = "lab-kit.toml")]
        config: PathBuf,
    },
    /// Build JSON (+ PDF if licensed) from HelixTest JSON output.
    Report {
        #[arg(long)]
        helixtest_json: PathBuf,
        #[arg(long, default_value = "reports/conformance")]
        out_dir: PathBuf,
        #[arg(short, long, default_value = "lab-kit.toml")]
        config: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Init { output } => init_wizard(&output).await?,
        Command::Generate { target } => match target {
            GenerateTarget::Compose {
                config,
                fragments,
                output,
                stdout,
            } => {
                let cfg =
                    load_config(&config).with_context(|| format!("load {}", config.display()))?;
                if stdout {
                    let tmp = tempfile::tempdir()?;
                    let p = tmp.path().join("out.yml");
                    generate_compose_file(&cfg, &fragments, &p)?;
                    print!("{}", std::fs::read_to_string(&p)?);
                } else {
                    generate_compose_file(&cfg, &fragments, &output)?;
                    tracing::info!(path = %output.display(), "wrote compose file");
                }
            }
            GenerateTarget::Helm {
                config,
                output,
                stdout,
            } => {
                let cfg =
                    load_config(&config).with_context(|| format!("load {}", config.display()))?;
                if stdout {
                    let tmp = tempfile::tempdir()?;
                    let p = tmp.path().join("values.yaml");
                    generate_helm_values(&cfg, &p)?;
                    print!("{}", std::fs::read_to_string(&p)?);
                } else {
                    generate_helm_values(&cfg, &output)?;
                    tracing::info!(path = %output.display(), "wrote helm values");
                }
            }
            GenerateTarget::Systemd { config, output_dir } => {
                let cfg =
                    load_config(&config).with_context(|| format!("load {}", config.display()))?;
                generate_systemd_units(&cfg, &output_dir)?;
                tracing::info!(dir = %output_dir.display(), "wrote systemd units");
            }
        },
        Command::Status { config } => {
            let cfg = load_config(&config).with_context(|| format!("load {}", config.display()))?;
            let reg = ServiceRegistry::from_config(&cfg);
            let health = lab_kit_core::HealthAggregator::poll(&reg)?;
            println!("{:#?}", health);
        }
        Command::Conformance { action } => match action {
            ConformanceAction::Run { config } => {
                let _cfg =
                    load_config(&config).with_context(|| format!("load {}", config.display()))?;
                let bin = std::env::var("HELIXTEST_BIN").unwrap_or_else(|_| "helixtest".into());
                tracing::info!(%bin, "invoke HelixTest from https://github.com/SynapticFour/HelixTest — not bundled");
                let status = std::process::Command::new(&bin).status();
                match status {
                    Ok(s) if s.success() => {}
                    Ok(s) => anyhow::bail!("HelixTest exited with {s}"),
                    Err(e) => anyhow::bail!(
                        "could not run {bin}: {e} — install HelixTest or set HELIXTEST_BIN"
                    ),
                }
            }
            ConformanceAction::Report {
                helixtest_json,
                out_dir,
                config,
            } => {
                let cfg =
                    load_config(&config).with_context(|| format!("load {}", config.display()))?;
                generate_reports(&helixtest_json, &out_dir, &cfg.lab.name)?;
                tracing::info!(dir = %out_dir.display(), "conformance reports generated");
            }
        },
        Command::Ferrum { action } => match action {
            FerrumAction::Check => {
                println!(
                    "Ferrum platform crate linked via lab-kit-ferrum → {}",
                    lab_kit_ferrum::ferrum_core_type_name()
                );
                println!(
                    "Pinned tag: {}  |  {}",
                    lab_kit_ferrum::FERRUM_GIT_TAG,
                    lab_kit_ferrum::FERRUM_GIT_URL
                );
                println!("Keep in sync: config/ci/ferrum-revision.txt");
            }
        },
    }
    Ok(())
}

async fn init_wizard(output: &Path) -> anyhow::Result<()> {
    let theme = ColorfulTheme::default();
    let name: String = Input::with_theme(&theme)
        .with_prompt("Lab / institute name")
        .interact_text()?;
    let contact: String = Input::with_theme(&theme)
        .with_prompt("Contact email (optional)")
        .allow_empty(true)
        .interact_text()?;
    let environment: String = Input::with_theme(&theme)
        .with_prompt("Environment")
        .default("production".into())
        .interact_text()?;

    let auth_idx = Select::with_theme(&theme)
        .with_prompt("Authentication provider")
        .items(&["ls-login (ELIXIR LS Login)", "none (demo only)"])
        .default(0)
        .interact()?;

    let (auth_section, ls_login) = if auth_idx == 0 {
        let client_id: String = Input::with_theme(&theme)
            .with_prompt("OIDC client_id")
            .interact_text()?;
        let client_secret: String = Input::with_theme(&theme)
            .with_prompt("OIDC client_secret")
            .interact_text()?;
        let issuer: String = Input::with_theme(&theme)
            .with_prompt("Issuer base URL")
            .default("https://login.elixir-czech.org/oidc/".into())
            .interact_text()?;
        (
            AuthSection {
                provider: AuthProviderKind::LsLogin,
                ls_login: Some(LsLoginConfig {
                    client_id,
                    client_secret,
                    issuer,
                    redirect_uri: None,
                    scopes: LsLoginOidc::default_scopes()
                        .into_iter()
                        .map(String::from)
                        .collect(),
                }),
                keycloak: None,
                ldap: None,
            },
            true,
        )
    } else {
        (
            AuthSection {
                provider: AuthProviderKind::None,
                ls_login: None,
                keycloak: None,
                ldap: None,
            },
            false,
        )
    };

    let service_labels = vec!["Beacon v2", "DRS", "WES", "TES", "TRS", "htsget"];
    let defaults = vec![true, false, false, false, false, false];
    let chosen = MultiSelect::with_theme(&theme)
        .with_prompt("Which GA4GH services should Lab Kit deploy?")
        .items(&service_labels)
        .defaults(&defaults)
        .interact()?;

    let mut services = ServicesSection::default();
    for i in chosen {
        match i {
            0 => {
                let dataset_id: String = Input::with_theme(&theme)
                    .with_prompt("Beacon dataset_id")
                    .interact_text()?;
                services.beacon = Some(BeaconServiceConfig {
                    external_url: None,
                    dataset_id,
                    access_level: BeaconAccessLevel::Registered,
                });
            }
            1 => services.drs = Some(Default::default()),
            2 => services.wes = Some(Default::default()),
            3 => services.tes = Some(Default::default()),
            4 => services.trs = Some(Default::default()),
            5 => services.htsget = Some(Default::default()),
            _ => {}
        }
    }

    if !ls_login && services.beacon.is_some() {
        let _ = Confirm::with_theme(&theme)
            .with_prompt("Beacon without LS Login is OK for public tiers only — continue?")
            .default(true)
            .interact()?;
    }

    let cfg = LabKitConfig {
        schema_version: 1,
        lab: LabSection {
            name,
            contact: if contact.is_empty() {
                None
            } else {
                Some(contact)
            },
            environment,
        },
        auth: auth_section,
        services,
        external: Default::default(),
    };
    cfg.validate()?;
    let toml = toml::to_string_pretty(&cfg)?;
    std::fs::write(output, toml)?;
    println!("Wrote {}", output.display());
    Ok(())
}
