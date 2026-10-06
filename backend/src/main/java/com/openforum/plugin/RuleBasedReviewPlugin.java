package com.openforum.plugin;

import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import com.openforum.entity.enums.ReviewMethod;
import org.springframework.stereotype.Component;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Set;

@Component
public class RuleBasedReviewPlugin implements ReviewPlugin {

    private static final Set<String> GENDERS = Set.of("male", "female", "other", "男", "女", "其他");

    @Override
    public String getKey() {
        return "rule-based-review";
    }

    @Override
    public String getName() {
        return "规则审核引擎";
    }

    @Override
    public PluginType getType() {
        return PluginType.REVIEW;
    }

    @Override
    public boolean isEnabledByDefault() {
        return true;
    }

    @Override
    public String getDescription() {
        return "校验各认证类型的必填字段与格式（身份证 18 位、年龄 0-120、国籍非空等）";
    }

    @Override
    public boolean isAvailable() {
        return true;
    }

    @Override
    public ReviewMethod getReviewMethod() {
        return ReviewMethod.RULE;
    }

    @Override
    public ReviewResult review(AuthType type, Map<String, Object> data) {
        List<String> errors = new ArrayList<>();
        switch (type) {
            case REAL_NAME -> requireText(data, "fullName", "真实姓名", 2, 50, errors);
            case ID_CARD -> validateIdCard(data, errors);
            case GENDER -> validateGender(data, errors);
            case AGE -> validateAge(data, errors);
            case NATIONALITY -> requireText(data, "nationality", "国籍", 1, 50, errors);
            case DRIVER_LICENSE -> requireText(data, "licenseNumber", "驾照号", 6, 40, errors);
            case MARRIAGE_CERT -> {
                requireText(data, "spouseName", "配偶姓名", 1, 50, errors);
                requireText(data, "certificateNumber", "结婚证号", 4, 40, errors);
            }
            case PROPERTY_CERT -> {
                requireText(data, "certificateNumber", "产权证号", 4, 60, errors);
                requireText(data, "propertyAddress", "房产地址", 2, 200, errors);
            }
            case ASSETS -> validateAssets(data, errors);
            case CUSTOM -> requireText(data, "description", "说明", 2, 500, errors);
        }
        if (errors.isEmpty()) {
            return new ReviewResult(AuthStatus.APPROVED, 0.95, "规则校验通过");
        }
        return new ReviewResult(AuthStatus.REJECTED, 0.95, String.join("；", errors));
    }

    private void validateIdCard(Map<String, Object> data, List<String> errors) {
        String value = text(data, "idNumber");
        if (value == null || value.isEmpty()) {
            errors.add("身份证号不能为空");
        } else if (!value.matches("^\\d{17}[\\dXx]$")) {
            errors.add("身份证号必须为 18 位（末位可为 X）");
        }
    }

    private void validateGender(Map<String, Object> data, List<String> errors) {
        String value = text(data, "gender");
        if (value == null || value.isEmpty()) {
            errors.add("性别不能为空");
        } else if (!GENDERS.contains(value.toLowerCase())) {
            errors.add("性别取值不合法");
        }
    }

    private void validateAge(Map<String, Object> data, List<String> errors) {
        String value = text(data, "age");
        if (value == null || value.isEmpty()) {
            errors.add("年龄不能为空");
            return;
        }
        try {
            int age = Integer.parseInt(value);
            if (age < 0 || age > 120) {
                errors.add("年龄需在 0-120 之间");
            }
        } catch (NumberFormatException ex) {
            errors.add("年龄必须为数字");
        }
    }

    private void validateAssets(Map<String, Object> data, List<String> errors) {
        String value = text(data, "amount");
        if (value == null || value.isEmpty()) {
            errors.add("资产金额不能为空");
            return;
        }
        try {
            double amount = Double.parseDouble(value);
            if (amount < 0) {
                errors.add("资产金额不能为负数");
            }
        } catch (NumberFormatException ex) {
            errors.add("资产金额必须为数字");
        }
    }

    private void requireText(Map<String, Object> data, String key, String label,
                             int min, int max, List<String> errors) {
        String value = text(data, key);
        if (value == null || value.isEmpty()) {
            errors.add(label + "不能为空");
        } else if (value.length() < min || value.length() > max) {
            errors.add(label + "长度需在 " + min + "-" + max + " 之间");
        }
    }

    private String text(Map<String, Object> data, String key) {
        if (data == null) {
            return null;
        }
        Object value = data.get(key);
        return value == null ? null : String.valueOf(value).trim();
    }
}
