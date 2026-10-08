package com.openforum.config;

import org.springframework.boot.context.properties.ConfigurationProperties;

@ConfigurationProperties(prefix = "openforum.google")
public record OpenForumGoogleProperties(String clientId) {

    public OpenForumGoogleProperties {
        if (clientId != null) {
            clientId = clientId.trim();
        }
    }

    public boolean isConfigured() {
        return clientId != null && !clientId.isBlank();
    }
}
