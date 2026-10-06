package com.openforum.dto.authentication;

import jakarta.validation.constraints.NotBlank;

import java.util.List;
import java.util.Map;

public record AuthenticationSubmitRequest(
        @NotBlank(message = "type 不能为空")
        String type,
        String level,
        Map<String, Object> data,
        List<Long> mediaIds,
        Long videoMediaId,
        String challengeId,
        Map<String, Object> liveness,
        Map<String, Object> attestation) {
}
