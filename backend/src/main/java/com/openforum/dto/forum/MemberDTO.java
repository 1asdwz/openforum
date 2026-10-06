package com.openforum.dto.forum;

import java.time.LocalDateTime;

public record MemberDTO(Long userId, String username, String avatar, String role, LocalDateTime joinedAt) {
}
