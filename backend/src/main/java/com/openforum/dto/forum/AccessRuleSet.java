package com.openforum.dto.forum;

import java.util.List;

/**
 * 板块访问门槛规则的 JSON 模型，存储在 sub_forums.access_rules 列中。
 *
 * <pre>
 * {
 *   "view":   { "groupLogic": "OR", "groups": [ { "innerLogic": "AND", "rules": [...] } ] },
 *   "action": { "groupLogic": "AND","groups": [ { "innerLogic": "AND", "rules": [...] } ] }
 * }
 * </pre>
 *
 * 支持的 field：gender | age | city | nationality | education | school | occupation |
 *               industry | company | property_count | property_city | has_vehicle |
 *               income_level | marital_status | interests | reg_days
 * <br>
 * 支持的 op：eq | ne | gte | lte | gt | lt | between | in | containsAny | containsAll
 */
public class AccessRuleSet {

    private RuleGroupSet view;
    private RuleGroupSet action;

    public AccessRuleSet() {
    }

    public AccessRuleSet(RuleGroupSet view, RuleGroupSet action) {
        this.view = view;
        this.action = action;
    }

    public RuleGroupSet getView() { return view; }
    public void setView(RuleGroupSet view) { this.view = view; }

    public RuleGroupSet getAction() { return action; }
    public void setAction(RuleGroupSet action) { this.action = action; }

    // ────────────────────────────────────────────────────────────────────────

    public static class RuleGroupSet {
        /** "AND"（默认）= 全部组满足；"OR" = 满足任一组即可 */
        private String groupLogic = "AND";
        private List<RuleGroup> groups = List.of();

        public RuleGroupSet() {
        }

        public String getGroupLogic() { return groupLogic; }
        public void setGroupLogic(String groupLogic) { this.groupLogic = groupLogic; }

        public List<RuleGroup> getGroups() { return groups; }
        public void setGroups(List<RuleGroup> groups) {
            this.groups = groups == null ? List.of() : groups;
        }

        public boolean isOr() { return "OR".equalsIgnoreCase(groupLogic); }

        public boolean isEmpty() {
            return groups == null || groups.isEmpty();
        }
    }

    public static class RuleGroup {
        /** "AND"（默认）= 组内全部满足；"OR" = 满足任一即可 */
        private String innerLogic = "AND";
        private List<Rule> rules = List.of();

        public RuleGroup() {
        }

        public String getInnerLogic() { return innerLogic; }
        public void setInnerLogic(String innerLogic) { this.innerLogic = innerLogic; }

        public List<Rule> getRules() { return rules; }
        public void setRules(List<Rule> rules) {
            this.rules = rules == null ? List.of() : rules;
        }

        public boolean isOr() { return "OR".equalsIgnoreCase(innerLogic); }
    }

    public static class Rule {
        /** 字段名，如 "gender", "age", "property_city" */
        private String field;
        /** 运算符，如 "eq", "gte", "between", "containsAny" */
        private String op;
        /**
         * 标量值（String/Number）或列表（Jackson 反序列化为 List&lt;Object&gt;）。
         * between 操作时忽略此字段，使用 min/max。
         */
        private Object value;
        /** between 操作的最小值 */
        private Integer min;
        /** between 操作的最大值 */
        private Integer max;
        /**
         * 该字段需要的最低认证等级，如 "LOW"（默认）/ "MEDIUM" / "HIGH" / "EXTREME"。
         * 用户认证等级不足时即使字段值满足也不通过。
         */
        private String minAuthLevel;

        public Rule() {
        }

        public String getField() { return field; }
        public void setField(String field) { this.field = field; }

        public String getOp() { return op; }
        public void setOp(String op) { this.op = op; }

        public Object getValue() { return value; }
        public void setValue(Object value) { this.value = value; }

        public Integer getMin() { return min; }
        public void setMin(Integer min) { this.min = min; }

        public Integer getMax() { return max; }
        public void setMax(Integer max) { this.max = max; }

        public String getMinAuthLevel() { return minAuthLevel; }
        public void setMinAuthLevel(String minAuthLevel) { this.minAuthLevel = minAuthLevel; }
    }
}
