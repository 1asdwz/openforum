package com.openforum.dto.user;

import java.time.LocalDateTime;
import java.util.List;

public record UserDTO(
        Long id,
        String username,
        String email,
        String avatar,
        String bio,
        String role,
        LocalDateTime createdAt,
        List<String> authentications) {
}
