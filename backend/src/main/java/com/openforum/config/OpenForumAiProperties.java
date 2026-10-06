package com.openforum.config;

import org.springframework.boot.context.properties.ConfigurationProperties;

@ConfigurationProperties(prefix = "openforum.ai")
public record OpenForumAiProperties(
        boolean enabled,
        String apiKey,
        String apiBase,
        String model,
        Integer timeoutSeconds) {

    public OpenForumAiProperties {
        if (apiBase == null || apiBase.isBlank()) {
            apiBase = "https://api.openai.com/v1";
        }
        if (model == null || model.isBlank()) {
            model = "gpt-4o-mini";
        }
        if (timeoutSeconds == null || timeoutSeconds <= 0) {
            timeoutSeconds = 20;
        }
    }

    public boolean isConfigured() {
        return enabled && apiKey != null && !apiKey.isBlank();
    }
}
