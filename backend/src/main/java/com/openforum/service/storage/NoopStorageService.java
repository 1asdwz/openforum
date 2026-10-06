package com.openforum.service.storage;

import com.openforum.exception.ApiException;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Service;

import java.io.InputStream;

/**
 * 未启用对象存储时的占位实现：任何写入/读取都拒绝，保证 MEDIUM+ 认证不会被误放行。
 */
@Service
@ConditionalOnProperty(prefix = "openforum.minio", name = "enabled", havingValue = "false", matchIfMissing = true)
public class NoopStorageService implements StorageService {

    @Override
    public boolean isEnabled() {
        return false;
    }

    @Override
    public void put(String objectKey, InputStream in, long size, String contentType) {
        throw ApiException.badRequest("文件存储未启用，无法上传认证材料");
    }

    @Override
    public InputStream get(String objectKey) {
        throw ApiException.badRequest("文件存储未启用");
    }

    @Override
    public void remove(String objectKey) {
    }
}
