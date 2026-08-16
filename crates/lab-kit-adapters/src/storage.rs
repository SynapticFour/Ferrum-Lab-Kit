// SPDX-License-Identifier: BUSL-1.1
use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("S3: {0}")]
    S3(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid object key: {0}")]
    InvalidKey(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageObjectMeta {
    pub key: String,
    pub size: u64,
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<(), StorageError>;
    async fn get_object(&self, key: &str) -> Result<Vec<u8>, StorageError>;
    async fn delete_object(&self, key: &str) -> Result<(), StorageError>;
    async fn head_object(&self, key: &str) -> Result<StorageObjectMeta, StorageError>;

    /// Stream a local file into the object store (override to avoid buffering).
    async fn put_file(&self, key: &str, path: &Path) -> Result<(), StorageError> {
        let data = tokio::fs::read(path).await?;
        self.put_object(key, &data).await
    }

    /// Stream an object to a local file (override to avoid buffering).
    async fn get_file(&self, key: &str, dest: &Path) -> Result<(), StorageError> {
        let data = self.get_object(key).await?;
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(dest, data).await?;
        Ok(())
    }
}
