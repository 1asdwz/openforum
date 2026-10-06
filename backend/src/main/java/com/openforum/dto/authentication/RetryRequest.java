package com.openforum.dto.authentication;

import java.util.List;
import java.util.Map;

public record RetryRequest(
        String level,
        Map<String, Object> data,
        List<Long> mediaIds,
        Long videoMediaId,
        String challengeId,
        Map<String, Object> liveness,
        Map<String, Object> attestation) {
}
