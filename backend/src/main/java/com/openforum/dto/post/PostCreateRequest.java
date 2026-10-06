package com.openforum.dto.post;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Size;

public record PostCreateRequest(
        @NotNull(message = "forumId 不能为空")
        Long forumId,
        @NotBlank(message = "标题不能为空")
        @Size(max = 300, message = "长度不能超过 300")
        String title,
        @NotBlank(message = "内容不能为空")
        String content) {
}
