package com.openforum.entity;

import com.openforum.entity.enums.CustomPageStatus;
import com.openforum.entity.enums.TemplateType;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.PrePersist;
import jakarta.persistence.PreUpdate;
import jakarta.persistence.Table;

import java.time.LocalDateTime;

@Entity
@Table(name = "sub_forums")
public class SubForum {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "slug", nullable = false, unique = true, length = 80)
    private String slug;

    @Column(name = "name", nullable = false, length = 120)
    private String name;

    @Column(name = "description", columnDefinition = "TEXT")
    private String description;

    @Column(name = "icon", length = 500)
    private String icon;

    @Column(name = "theme_color", length = 20)
    private String themeColor;

    @Column(name = "rules", columnDefinition = "TEXT")
    private String rules;

    @Column(name = "required_auth_types", length = 512)
    private String requiredAuthTypes;

    @Enumerated(EnumType.STRING)
    @Column(name = "template_type", nullable = false, length = 20)
    private TemplateType templateType = TemplateType.DEFAULT;

    @Column(name = "template_config", columnDefinition = "TEXT")
    private String templateConfig;

    @Column(name = "custom_page_html", columnDefinition = "LONGTEXT")
    private String customPageHtml;

    @Enumerated(EnumType.STRING)
    @Column(name = "custom_page_status", length = 20)
    private CustomPageStatus customPageStatus;

    @Column(name = "custom_page_review_note", length = 500)
    private String customPageReviewNote;

    @Column(name = "creator_id", nullable = false)
    private Long creatorId;

    @Column(name = "member_count", nullable = false)
    private int memberCount = 0;

    @Column(name = "post_count", nullable = false)
    private int postCount = 0;

    @Column(name = "is_public", nullable = false)
    private boolean isPublic = true;

    @Column(name = "created_at", nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @Column(name = "updated_at", nullable = false)
    private LocalDateTime updatedAt;

    @PrePersist
    void onCreate() {
        LocalDateTime now = LocalDateTime.now();
        if (createdAt == null) {
            createdAt = now;
        }
        updatedAt = now;
    }

    @PreUpdate
    void onUpdate() {
        updatedAt = LocalDateTime.now();
    }

    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }

    public String getSlug() {
        return slug;
    }

    public void setSlug(String slug) {
        this.slug = slug;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getDescription() {
        return description;
    }

    public void setDescription(String description) {
        this.description = description;
    }

    public String getIcon() {
        return icon;
    }

    public void setIcon(String icon) {
        this.icon = icon;
    }

    public String getThemeColor() {
        return themeColor;
    }

    public void setThemeColor(String themeColor) {
        this.themeColor = themeColor;
    }

    public String getRules() {
        return rules;
    }

    public void setRules(String rules) {
        this.rules = rules;
    }

    public String getRequiredAuthTypes() {
        return requiredAuthTypes;
    }

    public void setRequiredAuthTypes(String requiredAuthTypes) {
        this.requiredAuthTypes = requiredAuthTypes;
    }

    public TemplateType getTemplateType() {
        return templateType;
    }

    public void setTemplateType(TemplateType templateType) {
        this.templateType = templateType;
    }

    public String getTemplateConfig() {
        return templateConfig;
    }

    public void setTemplateConfig(String templateConfig) {
        this.templateConfig = templateConfig;
    }

    public String getCustomPageHtml() {
        return customPageHtml;
    }

    public void setCustomPageHtml(String customPageHtml) {
        this.customPageHtml = customPageHtml;
    }

    public CustomPageStatus getCustomPageStatus() {
        return customPageStatus;
    }

    public void setCustomPageStatus(CustomPageStatus customPageStatus) {
        this.customPageStatus = customPageStatus;
    }

    public String getCustomPageReviewNote() {
        return customPageReviewNote;
    }

    public void setCustomPageReviewNote(String customPageReviewNote) {
        this.customPageReviewNote = customPageReviewNote;
    }

    public Long getCreatorId() {
        return creatorId;
    }

    public void setCreatorId(Long creatorId) {
        this.creatorId = creatorId;
    }

    public int getMemberCount() {
        return memberCount;
    }

    public void setMemberCount(int memberCount) {
        this.memberCount = memberCount;
    }

    public int getPostCount() {
        return postCount;
    }

    public void setPostCount(int postCount) {
        this.postCount = postCount;
    }

    public boolean isPublic() {
        return isPublic;
    }

    public void setPublic(boolean isPublic) {
        this.isPublic = isPublic;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public void setCreatedAt(LocalDateTime createdAt) {
        this.createdAt = createdAt;
    }

    public LocalDateTime getUpdatedAt() {
        return updatedAt;
    }

    public void setUpdatedAt(LocalDateTime updatedAt) {
        this.updatedAt = updatedAt;
    }
}
