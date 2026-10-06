package com.openforum.entity.enums;

import com.openforum.exception.ApiException;

/**
 * 认证等级：与 {@link AuthType} 正交，表示「证据强度」。
 */
public enum AuthLevel {
    LOW(1, "低"),
    MEDIUM(2, "中"),
    HIGH(3, "高"),
    EXTREME(4, "极高");

    private final int rank;
    private final String label;

    AuthLevel(int rank, String label) {
        this.rank = rank;
        this.label = label;
    }

    public int rank() {
        return rank;
    }

    public String label() {
        return label;
    }

    /** 当前等级是否不低于 {@code other}。 */
    public boolean atLeast(AuthLevel other) {
        return other == null || this.rank >= other.rank;
    }

    public static AuthLevel parse(String value) {
        if (value == null || value.isBlank()) {
            return LOW;
        }
        try {
            return AuthLevel.valueOf(value.trim().toUpperCase());
        } catch (IllegalArgumentException ex) {
            throw ApiException.badRequest("未知认证等级: " + value);
        }
    }

    public static AuthLevel parseOrNull(String value) {
        if (value == null || value.isBlank()) {
            return null;
        }
        try {
            return AuthLevel.valueOf(value.trim().toUpperCase());
        } catch (IllegalArgumentException ex) {
            return null;
        }
    }

    /** 返回从 LOW 到当前等级的完整链路。 */
    public java.util.List<AuthLevel> ascending() {
        java.util.List<AuthLevel> all = new java.util.ArrayList<>();
        for (AuthLevel level : values()) {
            all.add(level);
            if (level == this) {
                break;
            }
        }
        return all;
    }
}
