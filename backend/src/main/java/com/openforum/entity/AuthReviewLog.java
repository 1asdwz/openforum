package com.openforum.entity;

import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.ReviewAction;
import com.openforum.entity.enums.ReviewMethod;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.PrePersist;
import jakarta.persistence.Table;

import java.time.LocalDateTime;

/**
 * 认证审计日志：记录每次提交与决策，保证升级/驳回可追溯。
 */
@Entity
@Table(name = "auth_review_logs")
public class AuthReviewLog {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "authentication_id", nullable = false)
    private Long authenticationId;

    @Enumerated(EnumType.STRING)
    @Column(name = "level", length = 20)
    private AuthLevel level;

    @Enumerated(EnumType.STRING)
    @Column(name = "action", length = 20)
    private ReviewAction action;

    @Enumerated(EnumType.STRING)
    @Column(name = "method", length = 20)
    private ReviewMethod method;

    @Column(name = "confidence")
    private Double confidence;

    @Column(name = "reason", length = 500)
    private String reason;

    @Column(name = "actor_id")
    private Long actorId;

    @Column(name = "created_at", nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @PrePersist
    void onCreate() {
        if (createdAt == null) {
            createdAt = LocalDateTime.now();
        }
    }

    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }

    public Long getAuthenticationId() {
        return authenticationId;
    }

    public void setAuthenticationId(Long authenticationId) {
        this.authenticationId = authenticationId;
    }

    public AuthLevel getLevel() {
        return level;
    }

    public void setLevel(AuthLevel level) {
        this.level = level;
    }

    public ReviewAction getAction() {
        return action;
    }

    public void setAction(ReviewAction action) {
        this.action = action;
    }

    public ReviewMethod getMethod() {
        return method;
    }

    public void setMethod(ReviewMethod method) {
        this.method = method;
    }

    public Double getConfidence() {
        return confidence;
    }

    public void setConfidence(Double confidence) {
        this.confidence = confidence;
    }

    public String getReason() {
        return reason;
    }

    public void setReason(String reason) {
        this.reason = reason;
    }

    public Long getActorId() {
        return actorId;
    }

    public void setActorId(Long actorId) {
        this.actorId = actorId;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public void setCreatedAt(LocalDateTime createdAt) {
        this.createdAt = createdAt;
    }
}
