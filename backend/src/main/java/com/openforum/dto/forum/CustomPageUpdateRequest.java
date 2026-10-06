package com.openforum.dto.forum;

import jakarta.validation.constraints.NotBlank;

public record CustomPageUpdateRequest(@NotBlank(message = "html 不能为空") String html) {
}
