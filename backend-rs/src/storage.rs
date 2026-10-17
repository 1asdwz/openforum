use s3::creds::Credentials;
use s3::{Bucket, Region};

use crate::config::Config;
use crate::error::ApiError;

/// 媒体字节存储抽象。生产环境为 MinIO 私有桶；未启用时所有写入/读取均拒绝。
///
/// Java 端使用流式读写，此处媒体上限 50MB，直接整块读入内存以简化实现。
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
        .map_err(|e| anyhow::anyhow!("MinIO 凭证无效: {e}"))?;
        let bucket = Bucket::new(&cfg.minio_bucket, region, credentials)
            .map_err(|e| anyhow::anyhow!("MinIO 初始化失败: {e}"))?
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
            Storage::Disabled => Err(ApiError::bad_request("文件存储未启用，无法上传认证材料")),
            Storage::Minio(minio) => minio
                .bucket
                .put_object_with_content_type(key, &bytes, content_type)
                .await
                .map(|_| ())
                .map_err(|e| {
                    tracing::error!("MinIO 上传失败: {e}");
                    ApiError::bad_request("文件上传失败，请稍后重试")
                }),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, ApiError> {
        match self {
            Storage::Disabled => Err(ApiError::bad_request("文件存储未启用")),
            Storage::Minio(minio) => minio
                .bucket
                .get_object(key)
                .await
                .map(|data| data.bytes().to_vec())
                .map_err(|e| {
                    tracing::error!("MinIO 读取失败: {e}");
                    ApiError::not_found("文件不存在或无法读取")
                }),
        }
    }
}
