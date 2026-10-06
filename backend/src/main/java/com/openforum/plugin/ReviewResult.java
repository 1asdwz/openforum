package com.openforum.plugin;

import com.openforum.entity.enums.AuthStatus;

import java.util.Map;

public record ReviewResult(AuthStatus status, double confidence, String reason, Map<String, Object> extracted) {

    public ReviewResult(AuthStatus status, double confidence, String reason) {
        this(status, confidence, reason, Map.of());
    }

    public ReviewResult {
        if (extracted == null) {
            extracted = Map.of();
        }
    }
}
