//! PostgreSQL metadata store using `sqlx` with embedded migrations.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use super::{
    ConformanceRunInsert, ConformanceRunRow, LicenseActivationRow, MetadataError, MetadataStore,
    ServiceRegistryRow,
};

/// Operational metadata in PostgreSQL (`service_registry`, `conformance_runs`, `license_activations`).
pub struct PostgresMetadataStore {
    pool: PgPool,
}

impl PostgresMetadataStore {
    /// Opens a pool, runs migrations from `migrations/postgres/`, and returns the store.
    ///
    /// # Reason
    /// Pool size is configurable for lab deployments; default `5` matches the Phase 2 spec.
    pub async fn connect(database_url: &str, max_connections: u32) -> Result<Self, MetadataError> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections.max(1))
            .connect(database_url)
            .await?;
        sqlx::migrate!("./migrations/postgres").run(&pool).await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl MetadataStore for PostgresMetadataStore {
    async fn ping(&self) -> Result<(), MetadataError> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    async fn upsert_service_registry_entry(
        &self,
        row: &ServiceRegistryRow,
    ) -> Result<(), MetadataError> {
        sqlx::query(
            r#"
            INSERT INTO service_registry (lab_name, service_name, endpoint_url, health_ok, last_health_check)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (lab_name, service_name) DO UPDATE SET
                endpoint_url = EXCLUDED.endpoint_url,
                health_ok = EXCLUDED.health_ok,
                last_health_check = EXCLUDED.last_health_check
            "#,
        )
        .bind(&row.lab_name)
        .bind(&row.service_name)
        .bind(&row.endpoint_url)
        .bind(row.health_ok)
        .bind(row.last_health_check)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_service_registry(
        &self,
        lab_name: Option<&str>,
    ) -> Result<Vec<ServiceRegistryRow>, MetadataError> {
        let rows = if let Some(lab) = lab_name {
            sqlx::query_as::<_, ServiceRegistryPgRow>(
                r#"SELECT lab_name, service_name, endpoint_url, health_ok, last_health_check
                   FROM service_registry WHERE lab_name = $1
                   ORDER BY service_name"#,
            )
            .bind(lab)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ServiceRegistryPgRow>(
                r#"SELECT lab_name, service_name, endpoint_url, health_ok, last_health_check
                   FROM service_registry ORDER BY lab_name, service_name"#,
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn insert_conformance_run(
        &self,
        row: &ConformanceRunInsert,
    ) -> Result<i64, MetadataError> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO conformance_runs (helix_output, overall_pass, per_service)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(row.helix_output.clone())
        .bind(row.overall_pass)
        .bind(row.per_service.clone())
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    async fn get_conformance_run(
        &self,
        id: i64,
    ) -> Result<Option<ConformanceRunRow>, MetadataError> {
        let row = sqlx::query_as::<_, ConformanceRunPgRow>(
            r#"SELECT id, run_at, helix_output, overall_pass, per_service
               FROM conformance_runs WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn list_conformance_runs(
        &self,
        limit: i64,
    ) -> Result<Vec<ConformanceRunRow>, MetadataError> {
        let lim = limit.clamp(1, 10_000);
        let rows = sqlx::query_as::<_, ConformanceRunPgRow>(
            r#"SELECT id, run_at, helix_output, overall_pass, per_service
               FROM conformance_runs ORDER BY run_at DESC LIMIT $1"#,
        )
        .bind(lim)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn upsert_license_activation(
        &self,
        row: &LicenseActivationRow,
    ) -> Result<(), MetadataError> {
        sqlx::query(
            r#"
            INSERT INTO license_activations (key_hash, activated_at, expires_at, features)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (key_hash) DO UPDATE SET
                activated_at = EXCLUDED.activated_at,
                expires_at = EXCLUDED.expires_at,
                features = EXCLUDED.features
            "#,
        )
        .bind(&row.key_hash)
        .bind(row.activated_at)
        .bind(row.expires_at)
        .bind(row.features.clone())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_license_by_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<LicenseActivationRow>, MetadataError> {
        let row = sqlx::query_as::<_, LicensePgRow>(
            r#"SELECT key_hash, activated_at, expires_at, features
               FROM license_activations WHERE key_hash = $1"#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }
}

#[derive(sqlx::FromRow)]
struct ServiceRegistryPgRow {
    lab_name: String,
    service_name: String,
    endpoint_url: Option<String>,
    health_ok: Option<bool>,
    last_health_check: Option<DateTime<Utc>>,
}

impl From<ServiceRegistryPgRow> for ServiceRegistryRow {
    fn from(r: ServiceRegistryPgRow) -> Self {
        ServiceRegistryRow {
            lab_name: r.lab_name,
            service_name: r.service_name,
            endpoint_url: r.endpoint_url,
            health_ok: r.health_ok,
            last_health_check: r.last_health_check,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ConformanceRunPgRow {
    id: i64,
    run_at: DateTime<Utc>,
    helix_output: serde_json::Value,
    overall_pass: bool,
    per_service: serde_json::Value,
}

impl From<ConformanceRunPgRow> for ConformanceRunRow {
    fn from(r: ConformanceRunPgRow) -> Self {
        ConformanceRunRow {
            id: r.id,
            run_at: r.run_at,
            helix_output: r.helix_output,
            overall_pass: r.overall_pass,
            per_service: r.per_service,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LicensePgRow {
    key_hash: String,
    activated_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    features: serde_json::Value,
}

impl From<LicensePgRow> for LicenseActivationRow {
    fn from(r: LicensePgRow) -> Self {
        LicenseActivationRow {
            key_hash: r.key_hash,
            activated_at: r.activated_at,
            expires_at: r.expires_at,
            features: r.features,
        }
    }
}
