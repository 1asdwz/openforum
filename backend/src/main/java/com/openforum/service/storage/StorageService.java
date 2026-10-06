package com.openforum.service.storage;

import java.io.InputStream;

/**
 * 认证媒体字节存储抽象（生产实现为 MinIO 私有桶）。
 */
public interface StorageService {

    boolean isEnabled();

    void put(String objectKey, InputStream in, long size, String contentType);

    /** 调用方负责关闭返回的流。 */
    InputStream get(String objectKey);

    void remove(String objectKey);
}
