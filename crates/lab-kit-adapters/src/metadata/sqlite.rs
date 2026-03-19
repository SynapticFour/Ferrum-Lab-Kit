//! SQLite metadata store using `sqlx` (same logical schema as PostgreSQL).

use std::str::FromStr as _;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use super::{
    ConformanceRunInsert, ConformanceRunRow, LicenseActivationRow, MetadataError, MetadataStore,
    ServiceRegistryRow,
};

/// File-backed or in-memory SQLite store for dev / single-node labs.
pub struct SqliteMetadataStore {
    pool: SqlitePool,
}

impl SqliteMetadataStore {
    /// Opens SQLite at `database_url` (e.g. `sqlite:data.db?mode=rwc` or `sqlite::memory:`).
    ///
    /// # Reason
    /// In-memory SQLite gives each connection its own empty DB unless a shared-cache URI is used.
    /// We cap the pool at **one** connection for `memory` URLs so inserts and reads always share
    /// the same database (see sqlx issue #362).
    pub async fn connect(database_url: &str) -> Result<Self, MetadataError> {
        let opts = SqliteConnectOptions::from_str(database_url)
            .map_err(|e| MetadataError::Other(e.to_string()))?;
        let max = if database_url.contains("memory") {
            1
        } else {
            5
        };
        let pool = SqlitePoolOptions::new()
            .max_connections(max)
            .connect_with(opts)
            .await?;
        sqlx::migrate!("./migrations/sqlite").run(&pool).await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl MetadataStore for SqliteMetadataStore {
    async fn ping(&self) -> Result<(), MetadataError> {
        sqlx::query_scalar::<_, i64>("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    async fn upsert_service_registry_entry(
        &self,
        row: &ServiceRegistryRow,
    ) -> Result<(), MetadataError> {
        let health: Option<i64> = row.health_ok.map(|b| if b { 1 } else { 0 });
        sqlx::query(
            r#"
            INSERT INTO service_registry (lab_name, service_name, endpoint_url, health_ok, last_health_check)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT (lab_name, service_name) DO UPDATE SET
                endpoint_url = excluded.endpoint_url,
                health_ok = excluded.health_ok,
                last_health_check = excluded.last_health_check
            "#,
        )
        .bind(&row.lab_name)
        .bind(&row.service_name)
        .bind(&row.endpoint_url)
        .bind(health)
        .bind(row.last_health_check.map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_service_registry(
        &self,
        lab_name: Option<&str>,
    ) -> Result<Vec<ServiceRegistryRow>, MetadataError> {
        let rows = if let Some(lab) = lab_name {
            sqlx::query_as::<_, ServiceRegistrySqliteRow>(
                r#"SELECT lab_name, service_name, endpoint_url, health_ok, last_health_check
                   FROM service_registry WHERE lab_name = ?
                   ORDER BY service_name"#,
            )
            .bind(lab)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ServiceRegistrySqliteRow>(
                r#"SELECT lab_name, service_name, endpoint_url, health_ok, last_health_check
                   FROM service_registry ORDER BY lab_name, service_name"#,
            )
            .fetch_all(&self.pool)
            .await?
        };
        rows.into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn insert_conformance_run(
        &self,
        row: &ConformanceRunInsert,
    ) -> Result<i64, MetadataError> {
        let helix = serde_json::to_string(&row.helix_output)?;
        let per = serde_json::to_string(&row.per_service)?;
        let res = sqlx::query(
            r#"
            INSERT INTO conformance_runs (helix_output, overall_pass, per_service)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(helix)
        .bind(if row.overall_pass { 1 } else { 0 })
        .bind(per)
        .execute(&self.pool)
        .await?;
        Ok(res.last_insert_rowid())
    }

    async fn get_conformance_run(
        &self,
        id: i64,
    ) -> Result<Option<ConformanceRunRow>, MetadataError> {
        let row = sqlx::query_as::<_, ConformanceRunSqliteRow>(
            r#"SELECT id, run_at, helix_output, overall_pass, per_service
               FROM conformance_runs WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn list_conformance_runs(
        &self,
        limit: i64,
    ) -> Result<Vec<ConformanceRunRow>, MetadataError> {
        let lim = limit.clamp(1, 10_000);
        let rows = sqlx::query_as::<_, ConformanceRunSqliteRow>(
            r#"SELECT id, run_at, helix_output, overall_pass, per_service
               FROM conformance_runs ORDER BY run_at DESC LIMIT ?"#,
        )
        .bind(lim)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn upsert_license_activation(
        &self,
        row: &LicenseActivationRow,
    ) -> Result<(), MetadataError> {
        let features = serde_json::to_string(&row.features)?;
        sqlx::query(
            r#"
            INSERT INTO license_activations (key_hash, activated_at, expires_at, features)
            VALUES (?, ?, ?, ?)
            ON CONFLICT (key_hash) DO UPDATE SET
                activated_at = excluded.activated_at,
                expires_at = excluded.expires_at,
                features = excluded.features
            "#,
        )
        .bind(&row.key_hash)
        .bind(row.activated_at.to_rfc3339())
        .bind(row.expires_at.map(|t| t.to_rfc3339()))
        .bind(features)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_license_by_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<LicenseActivationRow>, MetadataError> {
        let row = sqlx::query_as::<_, LicenseSqliteRow>(
            r#"SELECT key_hash, activated_at, expires_at, features
               FROM license_activations WHERE key_hash = ?"#,
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|r| r.try_into()).transpose()
    }
}

#[derive(sqlx::FromRow)]
struct ServiceRegistrySqliteRow {
    lab_name: String,
    service_name: String,
    endpoint_url: Option<String>,
    health_ok: Option<i64>,
    last_health_check: Option<String>,
}

impl TryFrom<ServiceRegistrySqliteRow> for ServiceRegistryRow {
    type Error = MetadataError;

    fn try_from(r: ServiceRegistrySqliteRow) -> Result<Self, Self::Error> {
        Ok(ServiceRegistryRow {
            lab_name: r.lab_name,
            service_name: r.service_name,
            endpoint_url: r.endpoint_url,
            health_ok: r.health_ok.map(|v| v != 0),
            last_health_check: parse_dt(r.last_health_check.as_deref())?,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ConformanceRunSqliteRow {
    id: i64,
    run_at: String,
    helix_output: String,
    overall_pass: i64,
    per_service: String,
}

impl TryFrom<ConformanceRunSqliteRow> for ConformanceRunRow {
    type Error = MetadataError;

    fn try_from(r: ConformanceRunSqliteRow) -> Result<Self, Self::Error> {
        Ok(ConformanceRunRow {
            id: r.id,
            run_at: parse_dt_required(Some(&r.run_at))?,
            helix_output: serde_json::from_str(&r.helix_output)?,
            overall_pass: r.overall_pass != 0,
            per_service: serde_json::from_str(&r.per_service)?,
        })
    }
}

#[derive(sqlx::FromRow)]
struct LicenseSqliteRow {
    key_hash: String,
    activated_at: String,
    expires_at: Option<String>,
    features: String,
}

impl TryFrom<LicenseSqliteRow> for LicenseActivationRow {
    type Error = MetadataError;

    fn try_from(r: LicenseSqliteRow) -> Result<Self, Self::Error> {
        Ok(LicenseActivationRow {
            key_hash: r.key_hash,
            activated_at: parse_dt_required(Some(&r.activated_at))?,
            expires_at: parse_dt(r.expires_at.as_deref())?,
            features: serde_json::from_str(&r.features)?,
        })
    }
}

fn parse_dt(s: Option<&str>) -> Result<Option<DateTime<Utc>>, MetadataError> {
    match s {
        None => Ok(None),
        Some("") => Ok(None),
        Some(t) => parse_dt_flex(t).map(Some),
    }
}

/// SQLite `datetime('now')` uses `YYYY-MM-DD HH:MM:SS`; we prefer RFC3339 from migrations and writers.
fn parse_dt_flex(t: &str) -> Result<DateTime<Utc>, MetadataError> {
    DateTime::parse_from_rfc3339(t)
        .map(|d| d.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S").map(|n| n.and_utc())
        })
        .map_err(|e| MetadataError::Other(e.to_string()))
}

fn parse_dt_required(s: Option<&str>) -> Result<DateTime<Utc>, MetadataError> {
    match s {
        None | Some("") => Err(MetadataError::Other("missing timestamp".into())),
        Some(t) => parse_dt_flex(t),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    #[tokio::test]
    async fn sqlite_memory_metadata_roundtrip() {
        let store = SqliteMetadataStore::connect("sqlite::memory:")
            .await
            .expect("connect");
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
            .expect("list");
        assert_eq!(listed.len(), 1);

        let run = ConformanceRunInsert {
            helix_output: json!({ "x": 1 }),
            overall_pass: false,
            per_service: json!({ "drs": "fail" }),
        };
        let id = store.insert_conformance_run(&run).await.expect("insert");
        let got = store
            .get_conformance_run(id)
            .await
            .expect("get")
            .expect("row");
        assert!(!got.overall_pass);

        let lic = LicenseActivationRow {
            key_hash: "h1".into(),
            activated_at: Utc::now(),
            expires_at: None,
            features: json!({}),
        };
        store
            .upsert_license_activation(&lic)
            .await
            .expect("license");
        assert!(store.get_license_by_hash("h1").await.unwrap().is_some());
    }
}
