package com.openforum.dto.forum;

import java.util.List;

public record ForumUpdateRequest(
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
