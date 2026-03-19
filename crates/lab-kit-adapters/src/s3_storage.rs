use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

use crate::storage::{StorageBackend, StorageError, StorageObjectMeta};

/// S3-compatible object store (AWS S3, MinIO, Ceph RGW).
pub struct S3StorageBackend {
    client: Client,
    bucket: String,
}

impl S3StorageBackend {
    /// Construct from explicit static credentials (`lab-kit.toml` style).
    pub async fn from_keys(
        bucket: impl Into<String>,
        endpoint: &str,
        region: &str,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Self, StorageError> {
        let creds = Credentials::new(access_key, secret_key, None, None, "lab-kit");
        let conf = aws_sdk_s3::config::Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(aws_config::Region::new(region.to_string()))
            .endpoint_url(endpoint)
            .force_path_style(true)
            .credentials_provider(creds)
            .build();
        let client = Client::from_conf(conf);
        Ok(Self {
            client,
            bucket: bucket.into(),
        })
    }
}

#[async_trait]
impl StorageBackend for S3StorageBackend {
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<(), StorageError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data.to_vec()))
            .send()
            .await
            .map_err(|e| StorageError::S3(e.to_string()))?;
        Ok(())
    }

    async fn get_object(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let out = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::S3(e.to_string()))?;
        let data = out
            .body
            .collect()
            .await
            .map_err(|e| StorageError::S3(e.to_string()))?
            .into_bytes();
        Ok(data.to_vec())
    }

    async fn delete_object(&self, key: &str) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::S3(e.to_string()))?;
        Ok(())
    }

    async fn head_object(&self, key: &str) -> Result<StorageObjectMeta, StorageError> {
        let out = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::S3(e.to_string()))?;
        Ok(StorageObjectMeta {
            key: key.to_string(),
            size: out.content_length().unwrap_or(0) as u64,
        })
    }
}
