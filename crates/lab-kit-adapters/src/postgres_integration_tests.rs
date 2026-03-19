//! Docker-backed Postgres checks for [`crate::metadata::PostgresMetadataStore`].
//!
//! Run locally (requires Docker):
//! `cargo test -p lab-kit-adapters --features integration-tests postgres_metadata_roundtrip`

use crate::metadata::{
    ConformanceRunInsert, LicenseActivationRow, MetadataStore, PostgresMetadataStore,
    ServiceRegistryRow,
};
use chrono::Utc;
use serde_json::json;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;

#[tokio::test]
async fn postgres_metadata_roundtrip() {
    let container = GenericImage::new("postgres", "16-alpine")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await
        .expect("start postgres container");

    let host = container.get_host().await.expect("container host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("mapped port");
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let store = PostgresMetadataStore::connect(&url, 5)
        .await
        .expect("connect with migrations");
    store.ping().await.expect("ping");

    let reg = ServiceRegistryRow {
        lab_name: "lab_a".into(),
        service_name: "drs".into(),
        endpoint_url: Some("https://drs.example".into()),
        health_ok: Some(true),
        last_health_check: Some(Utc::now()),
    };
    store
        .upsert_service_registry_entry(&reg)
        .await
        .expect("upsert registry");
    let listed = store
        .list_service_registry(Some("lab_a"))
        .await
        .expect("list registry");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].service_name, "drs");

    let run = ConformanceRunInsert {
        helix_output: json!({ "ok": true }),
        overall_pass: true,
        per_service: json!({}),
    };
    let id = store.insert_conformance_run(&run).await.expect("insert run");
    let got = store
        .get_conformance_run(id)
        .await
        .expect("get run")
        .expect("row exists");
    assert!(got.overall_pass);

    let lic = LicenseActivationRow {
        key_hash: "abc".into(),
        activated_at: Utc::now(),
        expires_at: None,
        features: json!({ "tier": "lab" }),
    };
    store
        .upsert_license_activation(&lic)
        .await
        .expect("upsert license");
    let lic2 = store
        .get_license_by_hash("abc")
        .await
        .expect("get license")
        .expect("row exists");
    assert_eq!(lic2.key_hash, "abc");
}
