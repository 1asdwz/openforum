package com.openforum.dto.comment;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

public record CommentCreateRequest(
        @NotNull(message = "postId 不能为空")
        Long postId,
        @NotBlank(message = "内容不能为空")
        String content,
        Long parentId) {
}
