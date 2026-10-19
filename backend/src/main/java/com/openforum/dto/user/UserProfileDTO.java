package com.openforum.dto.user;

import java.time.LocalDateTime;
import java.util.List;

/**
 * 用户扩展档案响应 DTO。age 由 birthYear 实时计算，不单独存储。
 */
public record UserProfileDTO(
        Long userId,
        String gender,
        Integer birthYear,
        Integer age,
        String nationality,
        String city,
        String education,
        String school,
        String major,
        String occupation,
        String industry,
        String company,
        String jobLevel,
        Integer propertyCount,
        List<String> propertyCities,
        Boolean hasVehicle,
        String incomeLevel,
        String maritalStatus,
        List<String> interests,
        LocalDateTime updatedAt) {
}
