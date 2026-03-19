//! Adapter traits for storage, compute, metadata, and workflow engines.
//! Ferrum services consume these boundaries; Lab Kit supplies defaults for common lab setups.

#![forbid(unsafe_code)]
// Public surface is consumed by Ferrum integrations; not all types are referenced in-tree yet.
#![allow(dead_code)]

mod compute;
mod metadata;
mod posix_storage;
#[cfg(feature = "s3")]
mod s3_storage;
mod slurm;
mod storage;
mod workflow;

#[cfg(all(test, feature = "integration-tests"))]
mod postgres_integration_tests;

pub use compute::{ComputeBackend, ComputeError, ComputeJobSpec, ComputeJobStatus};
pub use metadata::{
    ConformanceRunInsert, ConformanceRunRow, LicenseActivationRow, MetadataError, MetadataStore,
    PostgresMetadataStore, ServiceRegistryRow, SqliteMetadataStore,
};
pub use posix_storage::PosixStorageBackend;
#[cfg(feature = "s3")]
pub use s3_storage::S3StorageBackend;
pub use slurm::SlurmComputeBackend;
pub use storage::{StorageBackend, StorageError, StorageObjectMeta};
pub use workflow::{WorkflowEngine, WorkflowError, WorkflowRunSpec};
