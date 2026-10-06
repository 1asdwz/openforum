package com.openforum.dto.authentication;

import java.time.LocalDateTime;
import java.util.List;
import java.util.Map;

public record AuthenticationDTO(
        Long id,
        Long userId,
        String type,
        String typeLabel,
        String status,
        String reviewMethod,
        Double confidence,
        String reason,
        String rejectionReason,
        Map<String, Object> authData,
        LocalDateTime createdAt,
        LocalDateTime reviewedAt,
        String level,
        List<AuthMediaDTO> media,
        Map<String, Object> liveness,
        Map<String, Object> attestation) {
}
