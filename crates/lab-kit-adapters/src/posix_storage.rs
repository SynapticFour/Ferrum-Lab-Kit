// SPDX-License-Identifier: BUSL-1.1
use std::path::{Component, Path, PathBuf};

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

    /// Resolve `key` under `root`, rejecting `..`, absolute paths, and empty segments.
    /// Existing path prefixes are canonicalized so a symlink cannot escape `root`.
    fn key_path(&self, key: &str) -> Result<PathBuf, StorageError> {
        if key.is_empty() || key.contains('\0') {
            return Err(StorageError::InvalidKey(key.to_string()));
        }
        let rel = Path::new(key);
        if rel.is_absolute() {
            return Err(StorageError::InvalidKey(key.to_string()));
        }
        let root_canon = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        let mut acc = self.root.clone();
        for component in rel.components() {
            match component {
                Component::Normal(p) => acc.push(p),
                _ => return Err(StorageError::InvalidKey(key.to_string())),
            }
            if acc.exists() {
                let canon = acc.canonicalize()?;
                if !canon.starts_with(&root_canon) {
                    return Err(StorageError::InvalidKey(key.to_string()));
                }
            }
        }
        if !acc.starts_with(&self.root) && !acc.starts_with(&root_canon) {
            return Err(StorageError::InvalidKey(key.to_string()));
        }
        Ok(acc)
    }
}

#[async_trait]
impl StorageBackend for PosixStorageBackend {
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<(), StorageError> {
        let path = self.key_path(key)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn get_object(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let path = self.key_path(key)?;
        Ok(tokio::fs::read(path).await?)
    }

    async fn delete_object(&self, key: &str) -> Result<(), StorageError> {
        let path = self.key_path(key)?;
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    async fn head_object(&self, key: &str) -> Result<StorageObjectMeta, StorageError> {
        let path = self.key_path(key)?;
        let meta = tokio::fs::metadata(&path).await?;
        Ok(StorageObjectMeta {
            key: key.to_string(),
            size: meta.len(),
        })
    }

    async fn put_file(&self, key: &str, path: &Path) -> Result<(), StorageError> {
        let dest = self.key_path(key)?;
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::copy(path, dest).await?;
        Ok(())
    }

    async fn get_file(&self, key: &str, dest: &Path) -> Result<(), StorageError> {
        let src = self.key_path(key)?;
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::copy(src, dest).await?;
        Ok(())
    }
}

impl PosixStorageBackend {
    /// Exposed for synchronous tooling/tests.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_dotdot_keys() {
        let dir = tempfile::tempdir().unwrap();
        let store = PosixStorageBackend::new(dir.path());
        let err = store
            .put_object("../../etc/passwd", b"x")
            .await
            .unwrap_err();
        assert!(matches!(err, StorageError::InvalidKey(_)));
        let err = store.get_object("/etc/passwd").await.unwrap_err();
        assert!(matches!(err, StorageError::InvalidKey(_)));
        store.put_object("ok/file.bin", b"hi").await.unwrap();
        assert_eq!(store.get_object("ok/file.bin").await.unwrap(), b"hi");
    }
}
