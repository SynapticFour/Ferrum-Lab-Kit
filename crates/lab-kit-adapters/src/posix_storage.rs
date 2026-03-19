use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::storage::{StorageBackend, StorageError, StorageObjectMeta};

/// POSIX shared filesystem (NFS/Lustre) — common on DACH HPC.
pub struct PosixStorageBackend {
    root: PathBuf,
}

impl PosixStorageBackend {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn key_path(&self, key: &str) -> PathBuf {
        self.root.join(key.trim_start_matches('/'))
    }
}

#[async_trait]
impl StorageBackend for PosixStorageBackend {
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<(), StorageError> {
        let path = self.key_path(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn get_object(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let path = self.key_path(key);
        Ok(tokio::fs::read(path).await?)
    }

    async fn delete_object(&self, key: &str) -> Result<(), StorageError> {
        let path = self.key_path(key);
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    async fn head_object(&self, key: &str) -> Result<StorageObjectMeta, StorageError> {
        let path = self.key_path(key);
        let meta = tokio::fs::metadata(&path).await?;
        Ok(StorageObjectMeta {
            key: key.to_string(),
            size: meta.len(),
        })
    }
}

impl PosixStorageBackend {
    /// Exposed for synchronous tooling/tests.
    pub fn root(&self) -> &Path {
        &self.root
    }
}
