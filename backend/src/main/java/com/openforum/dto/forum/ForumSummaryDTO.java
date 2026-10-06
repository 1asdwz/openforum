package com.openforum.dto.forum;

import java.time.LocalDateTime;
import java.util.List;

public class ForumSummaryDTO {

    private final Long id;
    private final String slug;
    private final String name;
    private final String description;
    private final String icon;
    private final String themeColor;
    private final List<String> requiredAuthTypes;
    private final int memberCount;
    private final int postCount;
    private final boolean isMember;
    private final LocalDateTime createdAt;

    public ForumSummaryDTO(Long id, String slug, String name, String description, String icon, String themeColor,
                           List<String> requiredAuthTypes, int memberCount, int postCount, boolean isMember,
                           LocalDateTime createdAt) {
        this.id = id;
        this.slug = slug;
        this.name = name;
        this.description = description;
        this.icon = icon;
        this.themeColor = themeColor;
        this.requiredAuthTypes = requiredAuthTypes;
        this.memberCount = memberCount;
        this.postCount = postCount;
        this.isMember = isMember;
        this.createdAt = createdAt;
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

    public List<String> getRequiredAuthTypes() {
        return requiredAuthTypes;
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

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }
}
