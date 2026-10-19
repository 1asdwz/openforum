use s3::creds::Credentials;
use s3::{Bucket, Region};

use crate::config::Config;
use crate::error::ApiError;

/// Media byte storage abstraction. Production uses a private MinIO bucket; when disabled, all reads/writes are rejected.
///
/// The Java side streams reads and writes; here media is capped at 50MB and read into memory in one piece to keep things simple.
pub enum Storage {
    Disabled,
    Minio(MinioStorage),
}

pub struct MinioStorage {
    bucket: Box<Bucket>,
}

impl MinioStorage {
    pub fn new(cfg: &Config) -> anyhow::Result<Self> {
        let region = Region::Custom {
            region: "us-east-1".to_string(),
            endpoint: cfg.minio_endpoint.clone(),
        };
        let credentials = Credentials::new(
            Some(&cfg.minio_access_key),
            Some(&cfg.minio_secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| anyhow::anyhow!("Invalid MinIO credentials: {e}"))?;
        let bucket = Bucket::new(&cfg.minio_bucket, region, credentials)
            .map_err(|e| anyhow::anyhow!("MinIO initialization failed: {e}"))?
            .with_path_style();
        Ok(Self { bucket })
    }
}

impl Storage {
    pub fn from_config(cfg: &Config) -> anyhow::Result<Self> {
        if cfg.minio_enabled {
            Ok(Storage::Minio(MinioStorage::new(cfg)?))
        } else {
            Ok(Storage::Disabled)
        }
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self, Storage::Minio(_))
    }

    pub async fn put(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> Result<(), ApiError> {
        match self {
            Storage::Disabled => Err(ApiError::bad_request("File storage is disabled; cannot upload verification materials")),
            Storage::Minio(minio) => minio
                .bucket
                .put_object_with_content_type(key, &bytes, content_type)
                .await
                .map(|_| ())
                .map_err(|e| {
                    tracing::error!("MinIO upload failed: {e}");
                    ApiError::bad_request("File upload failed; please try again later")
                }),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, ApiError> {
        match self {
            Storage::Disabled => Err(ApiError::bad_request("File storage is disabled")),
            Storage::Minio(minio) => minio
                .bucket
                .get_object(key)
                .await
                .map(|data| data.bytes().to_vec())
                .map_err(|e| {
                    tracing::error!("MinIO read failed: {e}");
                    ApiError::not_found("File not found or unreadable")
                }),
        }
    }
}
