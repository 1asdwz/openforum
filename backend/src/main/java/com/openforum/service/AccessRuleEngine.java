package com.openforum.service;

import com.openforum.common.JsonSupport;
import com.openforum.dto.forum.AccessCheckResult;
import com.openforum.dto.forum.AccessCheckResult.GroupCheckResult;
import com.openforum.dto.forum.AccessCheckResult.RuleCheckResult;
import com.openforum.dto.forum.AccessRuleSet;
import com.openforum.dto.forum.AccessRuleSet.Rule;
import com.openforum.dto.forum.AccessRuleSet.RuleGroup;
import com.openforum.dto.forum.AccessRuleSet.RuleGroupSet;
import com.openforum.entity.User;
import com.openforum.entity.UserAuthentication;
import com.openforum.entity.UserProfile;
import com.openforum.entity.enums.AuthLevel;
import com.openforum.entity.enums.AuthStatus;
import com.openforum.entity.enums.AuthType;
import com.openforum.repository.UserAuthenticationRepository;
import com.openforum.repository.UserProfileRepository;
import com.openforum.repository.UserRepository;
import org.springframework.stereotype.Service;

import java.time.LocalDate;
import java.time.temporal.ChronoUnit;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

/**
 * 访问规则引擎：将 {@link AccessRuleSet}（JSON）与用户档案做实时匹配，
 * 返回携带每条规则满足状态的 {@link AccessCheckResult}，供前端精准引导。
 */
@Service
public class AccessRuleEngine {

    /** 学历等级排名：越高越严格 */
    private static final Map<String, Integer> EDUCATION_RANK = Map.of(
            "HIGHSCHOOL", 1, "ASSOCIATE", 2, "BACHELOR", 3, "MASTER", 4, "PHD", 5);

    /** 收入水平排名 */
    private static final Map<String, Integer> INCOME_RANK = Map.of(
            "LOW", 1, "MEDIUM", 2, "HIGH", 3);

    private final UserProfileRepository profileRepository;
    private final UserAuthenticationRepository authRepository;
    private final UserRepository userRepository;
    private final JsonSupport jsonSupport;

    public AccessRuleEngine(UserProfileRepository profileRepository,
                            UserAuthenticationRepository authRepository,
                            UserRepository userRepository,
                            JsonSupport jsonSupport) {
        this.profileRepository = profileRepository;
        this.authRepository = authRepository;
        this.userRepository = userRepository;
        this.jsonSupport = jsonSupport;
    }

    /** 解析 JSON 为 {@link AccessRuleSet}，解析失败返回 null。 */
    public AccessRuleSet parse(String json) {
        return jsonSupport.fromJson(json, AccessRuleSet.class);
    }

    /**
     * 对 userId 执行 accessRulesJson 描述的完整门槛检查。
     * 无规则时返回 {@link AccessCheckResult#noRules()}。
     */
    public AccessCheckResult check(Long userId, String accessRulesJson) {
        AccessRuleSet rules = parse(accessRulesJson);
        if (rules == null) {
            return AccessCheckResult.noRules();
        }

        // 加载用户档案与认证等级
        UserProfile profile = userId == null ? null :
                profileRepository.findByUserId(userId).orElse(null);
        Map<AuthType, AuthLevel> authLevels = userId == null ? Map.of() : loadApprovedLevels(userId);
        int regDays = resolveRegDays(userId);

        // 评估 view / action 两套规则
        List<GroupCheckResult> viewGroups = evalGroupSet(rules.getView(), profile, authLevels, regDays);
        List<GroupCheckResult> actionGroups = evalGroupSet(rules.getAction(), profile, authLevels, regDays);

        boolean canView = satisfies(rules.getView(), viewGroups);
        boolean canAct = satisfies(rules.getAction(), actionGroups);

        String viewMsg = canView ? null : buildMessage(viewGroups);
        String actMsg = canAct ? null : buildMessage(actionGroups);

        return new AccessCheckResult(canView, canAct, viewGroups, actionGroups, viewMsg, actMsg);
    }

    // ── 私有方法 ──────────────────────────────────────────────────────────────

    private Map<AuthType, AuthLevel> loadApprovedLevels(Long userId) {
        return authRepository.findByUserIdAndStatus(userId, AuthStatus.APPROVED)
                .stream()
                .collect(Collectors.toMap(
                        UserAuthentication::getType,
                        UserAuthentication::getLevel,
                        // 同一类型取等级更高的那条
                        (a, b) -> a.rank() >= b.rank() ? a : b));
    }

    private int resolveRegDays(Long userId) {
        if (userId == null) return 0;
        return userRepository.findById(userId)
                .map(User::getCreatedAt)
                .map(dt -> (int) ChronoUnit.DAYS.between(dt.toLocalDate(), LocalDate.now()))
                .orElse(0);
    }

    private List<GroupCheckResult> evalGroupSet(RuleGroupSet ruleGroupSet,
                                                UserProfile profile,
                                                Map<AuthType, AuthLevel> authLevels,
                                                int regDays) {
        if (ruleGroupSet == null || ruleGroupSet.isEmpty()) {
            return List.of();
        }
        return ruleGroupSet.getGroups().stream()
                .map(g -> evalGroup(g, profile, authLevels, regDays))
                .toList();
    }

    private GroupCheckResult evalGroup(RuleGroup group,
                                       UserProfile profile,
                                       Map<AuthType, AuthLevel> authLevels,
                                       int regDays) {
        if (group.getRules().isEmpty()) {
            return new GroupCheckResult(true, List.of());
        }
        List<RuleCheckResult> results = group.getRules().stream()
                .map(r -> evalRule(r, profile, authLevels, regDays))
                .toList();
        boolean passed = group.isOr()
                ? results.stream().anyMatch(RuleCheckResult::passed)
                : results.stream().allMatch(RuleCheckResult::passed);
        return new GroupCheckResult(passed, results);
    }

    private boolean satisfies(RuleGroupSet ruleGroupSet, List<GroupCheckResult> groupResults) {
        if (ruleGroupSet == null || groupResults.isEmpty()) return true;
        return ruleGroupSet.isOr()
                ? groupResults.stream().anyMatch(GroupCheckResult::passed)
                : groupResults.stream().allMatch(GroupCheckResult::passed);
    }

    private RuleCheckResult evalRule(Rule rule,
                                     UserProfile profile,
                                     Map<AuthType, AuthLevel> authLevels,
                                     int regDays) {
        String field = rule.getField();
        if (field == null) {
            return new RuleCheckResult("unknown", false, false, "规则字段未定义");
        }

        // 1. 先检查认证等级是否满足
        AuthType authType = fieldToAuthType(field);
        AuthLevel requiredLevel = AuthLevel.parseOrNull(rule.getMinAuthLevel());
        if (requiredLevel == null) requiredLevel = AuthLevel.LOW;

        if (authType != null && requiredLevel.rank() > AuthLevel.LOW.rank()) {
            AuthLevel userLevel = authLevels.get(authType);
            if (userLevel == null || !userLevel.atLeast(requiredLevel)) {
                return new RuleCheckResult(field, false, true,
                        "需要" + fieldLabel(field) + "达到" + requiredLevel.label() + "级认证");
            }
        }

        // 2. 检查字段值是否满足
        if (profile == null) {
            return new RuleCheckResult(field, false, false, "请先填写" + fieldLabel(field));
        }
        Object actual = resolveField(field, profile, regDays);
        if (actual == null) {
            return new RuleCheckResult(field, false, false, "请先填写" + fieldLabel(field));
        }

        boolean match = matchValue(rule, actual);
        return new RuleCheckResult(field, match, false, match ? null : describeExpected(rule));
    }

    private Object resolveField(String field, UserProfile profile, int regDays) {
        return switch (field.toLowerCase()) {
            case "gender"         -> profile.getGender();
            case "age"            -> profile.getBirthYear() == null ? null
                    : LocalDate.now().getYear() - profile.getBirthYear();
            case "city"           -> profile.getCity();
            case "nationality"    -> profile.getNationality();
            case "education"      -> profile.getEducation();
            case "school"         -> profile.getSchool();
            case "occupation"     -> profile.getOccupation();
            case "industry"       -> profile.getIndustry();
            case "company"        -> profile.getCompany();
            case "job_level"      -> profile.getJobLevel();
            case "property_count" -> profile.getPropertyCount();
            case "property_city"  -> jsonSupport.toStringList(profile.getPropertyCities());
            case "has_vehicle"    -> profile.getHasVehicle();
            case "income_level"   -> profile.getIncomeLevel();
            case "marital_status" -> profile.getMaritalStatus();
            case "interests"      -> jsonSupport.toStringList(profile.getInterests());
            case "reg_days"       -> regDays;
            default               -> null;
        };
    }

    @SuppressWarnings("unchecked")
    private boolean matchValue(Rule rule, Object actual) {
        String op = rule.getOp() == null ? "eq" : rule.getOp().toLowerCase();
        return switch (op) {
            case "eq", "="        -> valueEquals(actual, rule.getValue());
            case "ne", "!="       -> !valueEquals(actual, rule.getValue());
            case "gte", ">="      -> compare(actual, rule.getValue()) >= 0;
            case "lte", "<="      -> compare(actual, rule.getValue()) <= 0;
            case "gt", ">"        -> compare(actual, rule.getValue()) > 0;
            case "lt", "<"        -> compare(actual, rule.getValue()) < 0;
            case "between"        -> {
                int v = toInt(actual);
                yield (rule.getMin() == null || v >= rule.getMin())
                        && (rule.getMax() == null || v <= rule.getMax());
            }
            case "in"             -> {
                List<String> vals = toStringList(rule.getValue());
                yield vals.contains(String.valueOf(actual).toUpperCase());
            }
            case "containsany"    -> {
                List<String> haystack = actual instanceof List ? (List<String>) actual : List.of();
                List<String> needles = toStringList(rule.getValue());
                yield needles.stream().anyMatch(haystack::contains);
            }
            case "containsall"    -> {
                List<String> haystack = actual instanceof List ? (List<String>) actual : List.of();
                List<String> needles = toStringList(rule.getValue());
                yield haystack.containsAll(needles);
            }
            default               -> false;
        };
    }

    private boolean valueEquals(Object actual, Object expected) {
        if (actual == null || expected == null) return false;
        return String.valueOf(actual).equalsIgnoreCase(String.valueOf(expected));
    }

    private int compare(Object actual, Object expected) {
        // 尝试数值比较
        try {
            double a = Double.parseDouble(String.valueOf(actual));
            double e = Double.parseDouble(String.valueOf(expected));
            return Double.compare(a, e);
        } catch (NumberFormatException ignored) {
        }
        // 排名比较（学历/收入）
        Integer ar = rankValue(actual);
        Integer er = rankValue(expected);
        if (ar != null && er != null) return Integer.compare(ar, er);
        return String.valueOf(actual).compareToIgnoreCase(String.valueOf(expected));
    }

    private Integer rankValue(Object v) {
        if (v == null) return null;
        String s = String.valueOf(v).toUpperCase();
        Integer rank = EDUCATION_RANK.get(s);
        return rank != null ? rank : INCOME_RANK.get(s);
    }

    private int toInt(Object v) {
        if (v instanceof Number n) return n.intValue();
        try { return Integer.parseInt(String.valueOf(v)); } catch (NumberFormatException e) { return 0; }
    }

    @SuppressWarnings("unchecked")
    private List<String> toStringList(Object v) {
        if (v instanceof List<?> list) {
            return list.stream().map(Object::toString).toList();
        }
        if (v instanceof String s && s.startsWith("[")) {
            return jsonSupport.toStringList(s);
        }
        return v == null ? List.of() : List.of(String.valueOf(v));
    }

    private AuthType fieldToAuthType(String field) {
        return switch (field.toLowerCase()) {
            case "gender"                      -> AuthType.GENDER;
            case "age"                         -> AuthType.AGE;
            case "city"                        -> AuthType.CITY;
            case "nationality"                 -> AuthType.NATIONALITY;
            case "education", "school", "major"-> AuthType.EDUCATION;
            case "occupation", "industry",
                 "company", "job_level"        -> AuthType.OCCUPATION;
            case "property_count", "property_city" -> AuthType.PROPERTY_CERT;
            case "has_vehicle"                 -> AuthType.DRIVER_LICENSE;
            case "income_level"                -> AuthType.ASSETS;
            case "marital_status"              -> AuthType.MARRIAGE_CERT;
            case "interests"                   -> AuthType.INTERESTS;
            default                            -> null;
        };
    }

    private String fieldLabel(String field) {
        return switch (field.toLowerCase()) {
            case "gender"         -> "性别";
            case "age"            -> "年龄";
            case "city"           -> "所在城市";
            case "nationality"    -> "国籍";
            case "education"      -> "学历";
            case "school"         -> "院校";
            case "major"          -> "专业";
            case "occupation"     -> "职业";
            case "industry"       -> "行业";
            case "company"        -> "公司";
            case "job_level"      -> "职级";
            case "property_count" -> "房产数量";
            case "property_city"  -> "房产城市";
            case "has_vehicle"    -> "车辆";
            case "income_level"   -> "收入水平";
            case "marital_status" -> "婚姻状况";
            case "interests"      -> "兴趣爱好";
            case "reg_days"       -> "注册时长";
            default               -> field;
        };
    }

    private String describeExpected(Rule rule) {
        String field = fieldLabel(rule.getField());
        String op = rule.getOp() == null ? "eq" : rule.getOp().toLowerCase();
        return switch (op) {
            case "between"     -> field + " 需在 " + rule.getMin() + "–" + rule.getMax() + " 范围内";
            case "gte", ">="   -> field + " 需 ≥ " + rule.getValue();
            case "lte", "<="   -> field + " 需 ≤ " + rule.getValue();
            case "gt", ">"     -> field + " 需 > " + rule.getValue();
            case "lt", "<"     -> field + " 需 < " + rule.getValue();
            case "in", "containsany" -> field + " 需包含指定选项之一";
            case "containsall" -> field + " 需包含全部指定选项";
            default            -> field + " 不符合要求（期望: " + rule.getValue() + "）";
        };
    }

    private String buildMessage(List<GroupCheckResult> groupResults) {
        long failed = groupResults.stream().filter(g -> !g.passed()).count();
        return "有 " + failed + " 个条件组未满足，请补充资料后重试";
    }
}
