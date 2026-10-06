package com.openforum.dto.comment;

import java.time.LocalDateTime;
import java.util.List;

public record CommentDTO(
        Long id,
        Long postId,
        Long authorId,
        String authorName,
        String authorAvatar,
        Long parentId,
        String content,
        int upvotes,
        int downvotes,
        int score,
        LocalDateTime createdAt,
        List<CommentDTO> children) {
}
