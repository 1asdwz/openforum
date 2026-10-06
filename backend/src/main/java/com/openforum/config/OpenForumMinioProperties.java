package com.openforum.config;

import org.springframework.boot.context.properties.ConfigurationProperties;

@ConfigurationProperties(prefix = "openforum.minio")
public record OpenForumMinioProperties(
        boolean enabled,
        String endpoint,
        String accessKey,
        String secretKey,
        String bucket) {

    public OpenForumMinioProperties {
        if (endpoint == null || endpoint.isBlank()) {
            endpoint = "http://localhost:9000";
        }
        if (bucket == null || bucket.isBlank()) {
            bucket = "openforum-auth";
        }
    }

    public boolean isConfigured() {
        return enabled && endpoint != null && !endpoint.isBlank();
    }
}
