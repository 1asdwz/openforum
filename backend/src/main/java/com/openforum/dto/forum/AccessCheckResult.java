package com.openforum.dto.forum;

import java.util.List;

/**
 * 用户访问板块的权限检查结果，携带每个条件组和每条规则的满足状态，
 * 供前端精准展示「差哪一步」的引导信息。
 */
public record AccessCheckResult(
        boolean canView,
        boolean canAct,
        List<GroupCheckResult> viewGroups,
        List<GroupCheckResult> actionGroups,
        String viewMessage,
        String actionMessage) {

    /** 无门槛时返回此对象，一切允许。 */
    public static AccessCheckResult noRules() {
        return new AccessCheckResult(true, true, List.of(), List.of(), null, null);
    }

    /** 仅 canAct 受限时（有门槛但无 view 规则）。 */
    public static AccessCheckResult viewAllowed(boolean canAct,
                                                List<GroupCheckResult> actionGroups,
                                                String actionMessage) {
        return new AccessCheckResult(true, canAct, List.of(), actionGroups, null, actionMessage);
    }

    // ────────────────────────────────────────────────────────────────────────

    /** 单个条件组的检查结果 */
    public record GroupCheckResult(
            boolean passed,
            List<RuleCheckResult> rules) {
    }

    /** 单条规则的检查结果 */
    public record RuleCheckResult(
            String field,
            boolean passed,
            /** true = 字段值满足但认证等级不足 */
            boolean needsHigherAuth,
            /** 给用户看的提示，如「需要房产城市的高级认证」 */
            String hint) {
    }
}
