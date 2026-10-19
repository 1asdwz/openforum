package com.openforum.dto.forum;

import java.time.LocalDateTime;
import java.util.List;

public class ForumDetailDTO {

    private final Long id;
    private final String slug;
    private final String name;
    private final String description;
    private final String icon;
    private final String themeColor;
    private final String rules;
    private final List<String> requiredAuthTypes;
    private final String templateType;
    private final String templateConfig;
    private final String customPageStatus;
    private final String customPageHtml;
    private final int memberCount;
    private final int postCount;
    private final boolean isMember;
    private final boolean canPost;
    private final boolean isPublic;
    private final Long creatorId;
    private final String creatorName;
    private final LocalDateTime createdAt;
    /** 结构化访问门槛 JSON（供前端规则构建器回显） */
    private final String accessRules;
    /** 当前用户的门槛检查结果（null = 无门槛或未登录） */
    private final AccessCheckResult accessCheck;

    public ForumDetailDTO(Long id, String slug, String name, String description, String icon, String themeColor,
                          String rules, List<String> requiredAuthTypes, String templateType, String templateConfig,
                          String customPageStatus, String customPageHtml, int memberCount, int postCount,
                          boolean isMember, boolean canPost, boolean isPublic, Long creatorId, String creatorName,
                          LocalDateTime createdAt, String accessRules, AccessCheckResult accessCheck) {
        this.id = id;
        this.slug = slug;
        this.name = name;
        this.description = description;
        this.icon = icon;
        this.themeColor = themeColor;
        this.rules = rules;
        this.requiredAuthTypes = requiredAuthTypes;
        this.templateType = templateType;
        this.templateConfig = templateConfig;
        this.customPageStatus = customPageStatus;
        this.customPageHtml = customPageHtml;
        this.memberCount = memberCount;
        this.postCount = postCount;
        this.isMember = isMember;
        this.canPost = canPost;
        this.isPublic = isPublic;
        this.creatorId = creatorId;
        this.creatorName = creatorName;
        this.createdAt = createdAt;
        this.accessRules = accessRules;
        this.accessCheck = accessCheck;
    }

    public Long getId() {
        return id;
    }

    public String getSlug() {
        return slug;
    }

    public String getName() {
        return name;
    }

    public String getDescription() {
        return description;
    }

    public String getIcon() {
        return icon;
    }

    public String getThemeColor() {
        return themeColor;
    }

    public String getRules() {
        return rules;
    }

    public List<String> getRequiredAuthTypes() {
        return requiredAuthTypes;
    }

    public String getTemplateType() {
        return templateType;
    }

    public String getTemplateConfig() {
        return templateConfig;
    }

    public String getCustomPageStatus() {
        return customPageStatus;
    }

    public String getCustomPageHtml() {
        return customPageHtml;
    }

    public int getMemberCount() {
        return memberCount;
    }

    public int getPostCount() {
        return postCount;
    }

    public boolean getIsMember() {
        return isMember;
    }

    public boolean getCanPost() {
        return canPost;
    }

    public boolean getIsPublic() {
        return isPublic;
    }

    public Long getCreatorId() {
        return creatorId;
    }

    public String getCreatorName() {
        return creatorName;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public String getAccessRules() {
        return accessRules;
    }

    public AccessCheckResult getAccessCheck() {
        return accessCheck;
    }
}
