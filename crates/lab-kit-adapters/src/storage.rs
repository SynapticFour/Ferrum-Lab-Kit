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
}
