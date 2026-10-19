package com.openforum.service;

import com.openforum.dto.authentication.AuthLevelMetaDTO;
import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.AuthType;
import org.springframework.stereotype.Service;

import java.util.List;

/**
 * 「类型 × 等级」的说明事项与拍摄注意事项，是前端文案的唯一来源。
 */
@Service
public class AuthLevelCatalog {

    private static final List<String> TIPS = List.of(
            "请在光线充足、背景简洁的环境下拍摄",
            "面部保持无遮挡，请勿戴帽子、墨镜或口罩",
            "正对镜头并保持稳定，按屏幕提示依次完成动作",
            "证件需四角完整、文字清晰、无反光或遮挡");

    public List<AuthLevelMetaDTO> levelsFor(AuthType type) {
        String doc = documentName(type);
        return List.of(
                new AuthLevelMetaDTO(AuthLevel.LOW.name(), AuthLevel.LOW.label(),
                        List.of("填写" + label(type) + "相关信息"), List.of()),
                new AuthLevelMetaDTO(AuthLevel.MEDIUM.name(), AuthLevel.MEDIUM.label(),
                        List.of("填写" + label(type) + "相关信息",
                                "上传" + doc + "照片（正面清晰）",
                                "由 AI 审核材料真实性"),
                        TIPS),
                new AuthLevelMetaDTO(AuthLevel.HIGH.name(), AuthLevel.HIGH.label(),
                        List.of("填写" + label(type) + "相关信息",
                                "完成视频活体认证（随机动作序列 + 人脸分析）",
                                "上传" + doc + "照片",
                                "AI 审核通过后自动登记实名认证"),
                        TIPS),
                new AuthLevelMetaDTO(AuthLevel.EXTREME.name(), AuthLevel.EXTREME.label(),
                        List.of("完成「高」等级的全部步骤",
                                "使用 OpenForum 移动 App 防伪录制视频",
                                "通过后可获得最高可信度标识"),
                        TIPS));
    }

    private String documentName(AuthType type) {
        return switch (type) {
            case REAL_NAME -> "证件（身份证/护照）";
            case ID_CARD -> "身份证";
            case DRIVER_LICENSE -> "驾驶证";
            case MARRIAGE_CERT -> "结婚证";
            case PROPERTY_CERT -> "房产证";
            case NATIONALITY -> "国籍证明";
            case EDUCATION -> "学历证书（毕业证/学位证）";
            case OCCUPATION -> "在职证明或名片";
            case CITY -> "居住地址证明";
            case GENDER, AGE, ASSETS, INTERESTS, CUSTOM -> "相关证明材料";
        };
    }

    private String label(AuthType type) {
        return switch (type) {
            case REAL_NAME -> "实名";
            case ID_CARD -> "身份证";
            case GENDER -> "性别";
            case AGE -> "年龄";
            case NATIONALITY -> "国籍";
            case DRIVER_LICENSE -> "驾照";
            case MARRIAGE_CERT -> "婚姻";
            case PROPERTY_CERT -> "房产";
            case ASSETS -> "资产";
            case EDUCATION -> "学历";
            case OCCUPATION -> "职业";
            case CITY -> "城市";
            case INTERESTS -> "兴趣爱好";
            case CUSTOM -> "自定义";
        };
    }
}
