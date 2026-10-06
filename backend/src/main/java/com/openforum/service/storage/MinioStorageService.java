package com.openforum.service.storage;

import com.openforum.config.OpenForumMinioProperties;
import com.openforum.exception.ApiException;
import io.minio.BucketExistsArgs;
import io.minio.GetObjectArgs;
import io.minio.MakeBucketArgs;
import io.minio.MinioClient;
import io.minio.ObjectWriteArgs;
import io.minio.PutObjectArgs;
import io.minio.RemoveObjectArgs;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Service;

import java.io.InputStream;

/**
 * MinIO 私有桶实现。桶在首次使用时惰性创建，且不设置公开读策略。
 */
@Service
@ConditionalOnProperty(prefix = "openforum.minio", name = "enabled", havingValue = "true")
public class MinioStorageService implements StorageService {

    private static final Logger log = LoggerFactory.getLogger(MinioStorageService.class);

    private final OpenForumMinioProperties properties;
    private final MinioClient client;
    private volatile boolean bucketReady = false;

    public MinioStorageService(OpenForumMinioProperties properties) {
        this.properties = properties;
        this.client = MinioClient.builder()
                .endpoint(properties.endpoint())
                .credentials(properties.accessKey(), properties.secretKey())
                .build();
    }

    @Override
    public boolean isEnabled() {
        return true;
    }

    @Override
    public void put(String objectKey, InputStream in, long size, String contentType) {
        ensureBucket();
        try {
            PutObjectArgs args = PutObjectArgs.builder()
                    .bucket(properties.bucket())
                    .object(objectKey)
                    .contentType(contentType)
                    .stream(in, size, ObjectWriteArgs.MIN_MULTIPART_SIZE)
                    .build();
            client.putObject(args);
        } catch (Exception ex) {
            log.error("MinIO 上传失败: {}", ex.getMessage());
            throw ApiException.badRequest("文件上传失败，请稍后重试");
        }
    }

    @Override
    public InputStream get(String objectKey) {
        ensureBucket();
        try {
            return client.getObject(GetObjectArgs.builder()
                    .bucket(properties.bucket())
                    .object(objectKey)
                    .build());
        } catch (Exception ex) {
            log.error("MinIO 读取失败: {}", ex.getMessage());
            throw ApiException.notFound("文件不存在或无法读取");
        }
    }

    @Override
    public void remove(String objectKey) {
        try {
            client.removeObject(RemoveObjectArgs.builder()
                    .bucket(properties.bucket())
                    .object(objectKey)
                    .build());
        } catch (Exception ex) {
            log.warn("MinIO 删除失败: {}", ex.getMessage());
        }
    }

    private void ensureBucket() {
        if (bucketReady) {
            return;
        }
        synchronized (this) {
            if (bucketReady) {
                return;
            }
            try {
                boolean exists = client.bucketExists(BucketExistsArgs.builder()
                        .bucket(properties.bucket())
                        .build());
                if (!exists) {
                    client.makeBucket(MakeBucketArgs.builder()
                            .bucket(properties.bucket())
                            .build());
                    log.info("已创建私有桶: {}", properties.bucket());
                }
                bucketReady = true;
            } catch (Exception ex) {
                log.error("MinIO 桶初始化失败: {}", ex.getMessage());
                throw ApiException.badRequest("文件存储不可用，请稍后重试");
            }
        }
    }
}
