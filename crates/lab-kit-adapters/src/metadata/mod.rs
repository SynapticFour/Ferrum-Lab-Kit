//! Metadata persistence for Lab Kit operational state (not Ferrum’s application schema).

mod postgres;
mod sqlite;
mod types;

pub use postgres::PostgresMetadataStore;
pub use sqlite::SqliteMetadataStore;
pub use types::{
    ConformanceRunInsert, ConformanceRunRow, LicenseActivationRow, ServiceRegistryRow,
};

use async_trait::async_trait;
use thiserror::Error;

/// Errors from metadata stores (sqlx / serialization).
#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("database: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("migration: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

/// Lab Kit–owned tables: service registry, conformance history, license activations.
#[async_trait]
pub trait MetadataStore: Send + Sync {
    /// Verify database connectivity.
    async fn ping(&self) -> Result<(), MetadataError>;

    /// Upsert a service row keyed by `(lab_name, service_name)`.
    async fn upsert_service_registry_entry(&self, row: &ServiceRegistryRow)
        -> Result<(), MetadataError>;

    /// List registry rows, optionally filtered by `lab_name`.
    async fn list_service_registry(
        &self,
        lab_name: Option<&str>,
    ) -> Result<Vec<ServiceRegistryRow>, MetadataError>;

    /// Insert a conformance run; returns new row id.
    async fn insert_conformance_run(&self, row: &ConformanceRunInsert) -> Result<i64, MetadataError>;

    /// Fetch one conformance run by id.
    async fn get_conformance_run(&self, id: i64) -> Result<Option<ConformanceRunRow>, MetadataError>;

    /// Recent runs, newest first.
    async fn list_conformance_runs(&self, limit: i64) -> Result<Vec<ConformanceRunRow>, MetadataError>;

    /// Upsert license activation by `key_hash`.
    async fn upsert_license_activation(&self, row: &LicenseActivationRow) -> Result<(), MetadataError>;

    /// Lookup license by hash.
    async fn get_license_by_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<LicenseActivationRow>, MetadataError>;
}
