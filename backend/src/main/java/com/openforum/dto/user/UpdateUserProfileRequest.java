package com.openforum.dto.user;

import java.util.List;

/**
 * 更新用户扩展档案请求。所有字段均可选，null 表示不修改。
 */
public record UpdateUserProfileRequest(
        String gender,
        Integer birthYear,
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
        List<String> interests) {
}
