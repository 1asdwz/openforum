package com.openforum.dto.forum;

public record CustomPageDTO(
        Long forumId,
        String forumSlug,
        String forumName,
        String html,
        String status,
        String reviewNote) {
}
