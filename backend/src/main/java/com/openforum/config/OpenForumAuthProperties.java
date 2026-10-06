package com.openforum.config;

import org.springframework.boot.context.properties.ConfigurationProperties;

@ConfigurationProperties(prefix = "openforum.auth")
public record OpenForumAuthProperties(
        Integer challengeTtlMinutes,
        Integer maxDocumentCount,
        Long maxImageBytes,
        Long maxVideoBytes,
        Double autoApproveThreshold) {

    public OpenForumAuthProperties {
        if (challengeTtlMinutes == null || challengeTtlMinutes <= 0) {
            challengeTtlMinutes = 10;
        }
        if (maxDocumentCount == null || maxDocumentCount <= 0) {
            maxDocumentCount = 5;
        }
        if (maxImageBytes == null || maxImageBytes <= 0) {
            maxImageBytes = 10L * 1024 * 1024;
        }
        if (maxVideoBytes == null || maxVideoBytes <= 0) {
            maxVideoBytes = 50L * 1024 * 1024;
        }
        if (autoApproveThreshold == null || autoApproveThreshold <= 0) {
            autoApproveThreshold = 0.85;
        }
    }
}
