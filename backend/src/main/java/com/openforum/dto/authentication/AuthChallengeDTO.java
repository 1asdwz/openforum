package com.openforum.dto.authentication;

import java.time.LocalDateTime;
import java.util.List;

public record AuthChallengeDTO(String challengeId, List<String> actions, String nonce, LocalDateTime expiresAt) {
}
