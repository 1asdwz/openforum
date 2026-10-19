package com.openforum.dto.forum;

import jakarta.validation.constraints.NotBlank;

import java.util.List;

public record ForumCreateRequest(
        @NotBlank(message = "slug 不能为空")
        String slug,
        @NotBlank(message = "名称不能为空")
        String name,
        String description,
        String icon,
        String themeColor,
        String templateType,
        String templateConfig,
        String rules,
        List<String> requiredAuthTypes,
        /** 结构化访问门槛 JSON，优先级高于 requiredAuthTypes */
        String accessRules) {
}

