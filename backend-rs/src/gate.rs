use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::enums::AuthLevel;
use crate::error::ApiError;
use crate::info;
use crate::models::SubForum;

pub const GATE_BROWSE: &str = "BROWSE";
pub const GATE_OPERATE: &str = "OPERATE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateKind {
    Browse,
    Operate,
}

impl GateKind {
    pub fn code(self) -> &'static str {
        match self {
            GateKind::Browse => GATE_BROWSE,
            GateKind::Operate => GATE_OPERATE,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            GateKind::Browse => "浏览",
            GateKind::Operate => "操作",
        }
    }
}

/// 板块门槛规则。空条件表示无门槛。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GateRule {
    #[serde(default)]
    pub version: i32,
    #[serde(default)]
    pub conditions: Vec<GateCondition>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GateCondition {
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    /// 与 `values` 一一对应的展示名称，仅用于生成可读提示。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_level: Option<String>,
}

impl GateRule {
    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{\"version\":1,\"conditions\":[]}".to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingItem {
    pub field_key: String,
    pub label: String,
    pub reason: &'static str,
    pub required: String,
}

#[derive(Debug, Clone, Default)]
pub struct GateVerdict {
    pub passed: bool,
    pub login_required: bool,
    /// 旧规则回退时的可读提示。
    pub message: Option<String>,
    pub missing: Vec<MissingItem>,
}

impl GateVerdict {
    pub fn allow() -> Self {
        Self {
            passed: true,
            ..Default::default()
        }
    }

    pub fn deny(&self, kind: GateKind, forum: &SubForum) -> ApiError {
        let message = if self.login_required {
            "请先登录后再访问该板块".to_string()
        } else if let Some(msg) = &self.message {
            msg.clone()
        } else {
            format!("你还未满足本板块的{}门槛", kind.label())
        };
        ApiError::forbidden(message).with_data(json!({
            "gate": kind.code(),
            "forumId": forum.id,
            "forumName": forum.name,
            "loginRequired": self.login_required,
            "missing": self.missing,
        }))
    }
}

// -------- 规则解析与校验 --------

pub fn parse_rule_text(raw: Option<&str>) -> Option<GateRule> {
    let text = raw.map(|s| s.trim()).filter(|s| !s.is_empty())?;
    match serde_json::from_str::<GateRule>(text) {
        Ok(rule) => Some(rule),
        Err(e) => {
            tracing::warn!("门槛规则 JSON 解析失败（忽略）: {e}");
            None
        }
    }
}

fn bad(index: usize, message: impl std::fmt::Display) -> ApiError {
    ApiError::bad_request(format!("第 {} 个门槛条件：{message}", index + 1))
}

/// 校验并规范化前端提交的规则。`None`/`null` 表示未设置。
pub fn normalize_rule(value: Option<&Value>) -> Result<Option<GateRule>, ApiError> {
    let value = match value {
        None | Some(Value::Null) => return Ok(None),
        Some(v) => v,
    };
    let mut rule: GateRule = serde_json::from_value(value.clone())
        .map_err(|e| ApiError::bad_request(format!("门槛规则格式不正确: {e}")))?;
    if rule.version == 0 {
        rule.version = 1;
    }
    if rule.version != 1 {
        return Err(ApiError::bad_request("不支持的门槛规则版本"));
    }
    let mut conditions = Vec::with_capacity(rule.conditions.len());
    for (index, cond) in rule.conditions.iter().enumerate() {
        conditions.push(normalize_condition(index, cond)?);
    }
    rule.conditions = conditions;
    Ok(Some(rule))
}

fn normalize_condition(index: usize, cond: &GateCondition) -> Result<GateCondition, ApiError> {
    let field = info::find_field(&cond.field)
        .ok_or_else(|| bad(index, format!("未知信息项 {}", cond.field)))?;
    let op = cond
        .op
        .as_deref()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty());
    let auth_level = cond
        .auth_level
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    if op.is_none() && auth_level.is_none() {
        return Err(bad(index, "至少需要一个取值条件或认证等级要求"));
    }
    if let Some(op) = &op {
        let allowed = info::gate_ops(field);
        if !allowed.contains(&op.as_str()) {
            return Err(bad(index, format!("字段 {} 不支持算子 {}", field.key, op)));
        }
    }
    let auth_level = match auth_level {
        None => None,
        Some(raw) => {
            if field.key == "INTERESTS" {
                return Err(bad(index, "兴趣标签不支持认证等级"));
            }
            if !field.authable {
                return Err(bad(index, format!("字段 {} 不支持认证等级", field.key)));
            }
            Some(AuthLevel::parse(raw).map_err(|_| bad(index, format!("未知认证等级 {raw}")))?)
        }
    };

    let mut out = GateCondition {
        field: field.key.to_string(),
        op: op.clone(),
        auth_level: auth_level.map(|l| l.name().to_string()),
        ..Default::default()
    };

    match op.as_deref() {
        None => {}
        Some("in") | Some("not_in") => {
            let raw_values = cond.values.as_deref().unwrap_or(&[]);
            let mut values: Vec<String> = Vec::new();
            for raw in raw_values {
                let v = normalize_gate_value(field.key, raw)
                    .ok_or_else(|| bad(index, format!("{} 的取值 {raw} 不合法", field.label)))?;
                if !values.contains(&v) {
                    values.push(v);
                }
            }
            if values.is_empty() {
                return Err(bad(index, "请至少选择一个取值"));
            }
            out.labels = align_labels(cond.labels.as_deref(), &values);
            out.values = Some(values);
        }
        Some("eq") | Some("gte") | Some("lte") => {
            if matches!(field.key, "AGE" | "PROPERTY_COUNT") {
                let min = cond
                    .min
                    .or_else(|| cond.value.as_deref().and_then(|v| v.trim().parse().ok()));
                let max = cond
                    .max
                    .or_else(|| cond.value.as_deref().and_then(|v| v.trim().parse().ok()));
                match op.as_deref() {
                    Some("gte") if min.is_none() => {
                        return Err(bad(index, "请填写下限"))
                    }
                    Some("lte") if max.is_none() => {
                        return Err(bad(index, "请填写上限"))
                    }
                    Some("eq") => {
                        if min.is_none() && max.is_none() {
                            return Err(bad(index, "请填写取值"));
                        }
                    }
                    _ => {}
                }
                out.min = min;
                out.max = max;
            } else if field.key == "EDUCATION" {
                let raw = cond.value.as_deref().unwrap_or("");
                let edu = info::education_rank(raw)
                    .map(|_| raw.trim().to_ascii_uppercase())
                    .ok_or_else(|| bad(index, "学历取值不合法"))?;
                out.value = Some(edu);
            } else {
                let raw = cond.value.as_deref().unwrap_or("").trim();
                if raw.is_empty() {
                    return Err(bad(index, "请填写取值"));
                }
                out.value = Some(raw.to_string());
            }
        }
        Some("between") => {
            let (min, max) = match (cond.min, cond.max) {
                (Some(min), Some(max)) if min <= max => (min, max),
                (Some(_), Some(_)) => return Err(bad(index, "下限不能大于上限")),
                _ => return Err(bad(index, "区间条件需要同时填写下限和上限")),
            };
            out.min = Some(min);
            out.max = Some(max);
        }
        Some(_) => {}
    }
    Ok(out)
}

fn normalize_gate_value(field_key: &str, raw: &str) -> Option<String> {
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }
    match field_key {
        "GENDER" => info::normalize_gender(text).map(|s| s.to_string()),
        "COUNTRY" => info::country_code(text),
        "OCCUPATION" => {
            let up = text.to_ascii_uppercase();
            info::is_valid_occupation(&up).then_some(up)
        }
        "EDUCATION" => {
            let up = text.to_ascii_uppercase();
            info::education_rank(&up).map(|_| up)
        }
        "INTERESTS" if text.chars().count() > 30 => None,
        _ => Some(text.to_string()),
    }
}

fn align_labels(labels: Option<&[String]>, values: &[String]) -> Option<Vec<String>> {
    let labels = labels?;
    if labels.len() != values.len() {
        return None;
    }
    let cleaned: Vec<String> = labels
        .iter()
        .map(|l| l.trim())
        .map(|l| {
            if l.is_empty() {
                String::new()
            } else {
                l.to_string()
            }
        })
        .collect();
    if cleaned.iter().all(|l| l.is_empty()) {
        None
    } else {
        Some(cleaned)
    }
}

// -------- 用户上下文 --------

#[derive(Debug, Clone, Default)]
pub struct InfoRow {
    pub value: Value,
    pub auth_level: Option<String>,
    pub auth_status: Option<String>,
}

impl InfoRow {
    fn auth_rank(&self) -> i32 {
        AuthLevel::parse_or_null(self.auth_level.as_deref())
            .unwrap_or(AuthLevel::Low)
            .rank()
    }

    fn approved(&self) -> bool {
        self.auth_status.as_deref() == Some("APPROVED")
    }
}

pub struct GateContext {
    pub user_id: Option<i64>,
    pub is_admin: bool,
    pub manager_forums: HashSet<i64>,
    pub info: HashMap<String, InfoRow>,
}

fn db_code(e: &sqlx::Error) -> Option<String> {
    match e {
        sqlx::Error::Database(db) => db.code().map(|c| c.to_string()),
        _ => None,
    }
}

pub async fn load_context(
    pool: &MySqlPool,
    user_id: Option<i64>,
    is_admin: bool,
) -> Result<GateContext, ApiError> {
    let mut manager_forums = HashSet::new();
    let mut info = HashMap::new();
    if let Some(uid) = user_id {
        let rows: Vec<(i64,)> = sqlx::query_as(
            "SELECT forum_id FROM forum_members WHERE user_id = ? AND role IN ('OWNER','MODERATOR')",
        )
        .bind(uid)
        .fetch_all(pool)
        .await?;
        manager_forums.extend(rows.into_iter().map(|(f,)| f));

        match sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>)>(
            "SELECT field_key, value_json, auth_level, auth_status FROM user_info WHERE user_id = ?",
        )
        .bind(uid)
        .fetch_all(pool)
        .await
        {
            Ok(rows) => {
                for (field_key, value_json, auth_level, auth_status) in rows {
                    let value = value_json
                        .as_deref()
                        .and_then(|s| serde_json::from_str::<Value>(s).ok())
                        .unwrap_or(Value::Null);
                    info.insert(
                        field_key.to_ascii_uppercase(),
                        InfoRow {
                            value,
                            auth_level,
                            auth_status,
                        },
                    );
                }
            }
            Err(e) if db_code(&e).as_deref() == Some("1146") => {
                tracing::warn!("user_info 表不存在，门槛按旧逻辑回退");
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(GateContext {
        user_id,
        is_admin,
        manager_forums,
        info,
    })
}

impl GateContext {
    pub fn exempt(&self, forum: &SubForum) -> bool {
        if self.is_admin {
            return true;
        }
        match self.user_id {
            Some(uid) => uid == forum.creator_id || self.manager_forums.contains(&forum.id),
            None => false,
        }
    }

    pub fn evaluate(&self, conditions: &[GateCondition]) -> Vec<MissingItem> {
        conditions
            .iter()
            .filter_map(|cond| self.condition_missing(cond))
            .collect()
    }

    fn condition_missing(&self, cond: &GateCondition) -> Option<MissingItem> {
        let field = match info::find_field(&cond.field) {
            Some(f) => f,
            None => {
                tracing::warn!("门槛引用了未知信息项 {}，已跳过", cond.field);
                return None;
            }
        };
        let row = self.info.get(field.key);
        let filled = row.map(|r| info::is_filled(&r.value)).unwrap_or(false);
        if !filled {
            return Some(MissingItem {
                field_key: field.key.to_string(),
                label: field.label.to_string(),
                reason: "NOT_FILLED",
                required: requirement_text(field, cond),
            });
        }
        let row = row.expect("filled implies row");

        if let Some(op) = cond.op.as_deref() {
            if !value_matches(field.key, op, cond, &row.value) {
                return Some(MissingItem {
                    field_key: field.key.to_string(),
                    label: field.label.to_string(),
                    reason: "VALUE_MISMATCH",
                    required: requirement_text(field, cond),
                });
            }
        }

        if let Some(required) = cond.auth_level.as_deref() {
            let need = AuthLevel::parse_or_null(Some(required)).unwrap_or(AuthLevel::Low);
            if !row.approved() {
                let reason = if row.auth_status.as_deref() == Some("PENDING") {
                    "AUTH_PENDING"
                } else {
                    "AUTH_LEVEL_LOW"
                };
                return Some(MissingItem {
                    field_key: field.key.to_string(),
                    label: field.label.to_string(),
                    reason,
                    required: requirement_text(field, cond),
                });
            }
            if row.auth_rank() < need.rank() {
                return Some(MissingItem {
                    field_key: field.key.to_string(),
                    label: field.label.to_string(),
                    reason: "AUTH_LEVEL_LOW",
                    required: requirement_text(field, cond),
                });
            }
        }
        None
    }
}

fn value_matches(field_key: &str, op: &str, cond: &GateCondition, value: &Value) -> bool {
    match op {
        "in" => matches_any(field_key, value, cond.values.as_deref().unwrap_or(&[])),
        "not_in" => !matches_any(field_key, value, cond.values.as_deref().unwrap_or(&[])),
        "between" => {
            let actual = numeric_of(field_key, value);
            match (actual, cond.min, cond.max) {
                (Some(actual), Some(min), Some(max)) => actual >= min && actual <= max,
                _ => false,
            }
        }
        "gte" => compare_ordinal(field_key, value, cond, |actual, bound| actual >= bound),
        "lte" => compare_ordinal(field_key, value, cond, |actual, bound| actual <= bound),
        "eq" => compare_ordinal(field_key, value, cond, |actual, bound| actual == bound),
        _ => true,
    }
}

fn compare_ordinal(
    field_key: &str,
    value: &Value,
    cond: &GateCondition,
    cmp: impl Fn(f64, f64) -> bool,
) -> bool {
    if field_key == "EDUCATION" {
        let actual = value
            .as_object()
            .and_then(|m| m.get("education"))
            .and_then(|v| v.as_str())
            .and_then(info::education_rank);
        let bound = cond.value.as_deref().and_then(info::education_rank);
        return match (actual, bound) {
            (Some(a), Some(b)) => cmp(a as f64, b as f64),
            _ => false,
        };
    }
    let actual = numeric_of(field_key, value);
    let bound = if matches!(field_key, "AGE" | "PROPERTY_COUNT") {
        cond.min.or(cond.max)
    } else {
        cond.value.as_deref().and_then(|v| v.trim().parse::<f64>().ok())
    };
    match (actual, bound) {
        (Some(a), Some(b)) => cmp(a, b),
        _ => false,
    }
}

fn numeric_of(field_key: &str, value: &Value) -> Option<f64> {
    match field_key {
        "AGE" => info::age_of(value).map(|a| a as f64),
        "PROPERTY_COUNT" => value
            .as_object()
            .and_then(|m| m.get("count"))
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
            }),
        _ => None,
    }
}

fn matches_any(field_key: &str, value: &Value, values: &[String]) -> bool {
    if values.is_empty() {
        return false;
    }
    let map = match value.as_object() {
        Some(m) => m,
        None => return false,
    };
    let text = |key: &str| {
        map.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };
    match field_key {
        "GENDER" => {
            let actual = info::normalize_gender(&text("gender")).unwrap_or("");
            values.iter().any(|v| {
                info::normalize_gender(v)
                    .map(|g| g == actual)
                    .unwrap_or(false)
            })
        }
        "COUNTRY" => {
            let actual = text("country").to_ascii_uppercase();
            values.iter().any(|v| {
                info::country_code(v)
                    .map(|c| c == actual)
                    .unwrap_or(false)
            })
        }
        "CITY" => {
            let actual = text("city");
            !actual.is_empty() && values.iter().any(|v| v == &actual)
        }
        "OCCUPATION" => {
            let actual = text("category").to_ascii_uppercase();
            values.iter().any(|v| v.eq_ignore_ascii_case(&actual))
        }
        "INTERESTS" => {
            let tags = match map.get("tags") {
                Some(Value::Array(items)) => items,
                _ => return false,
            };
            tags.iter()
                .filter_map(|v| v.as_str())
                .any(|tag| values.iter().any(|v| v == tag))
        }
        "PROPERTY_AREA" => {
            let mut actual: Vec<String> = Vec::new();
            for key in ["cities", "districts"] {
                if let Some(Value::Array(items)) = map.get(key) {
                    actual.extend(
                        items
                            .iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string()),
                    );
                }
            }
            actual.iter().any(|v| values.contains(v))
        }
        _ => false,
    }
}

// -------- 可读提示 --------

fn auth_level_label(level: &str) -> &'static str {
    match level.trim().to_ascii_uppercase().as_str() {
        "MEDIUM" => "中",
        "HIGH" => "高",
        "EXTREME" => "极高",
        _ => "低",
    }
}

fn value_label(index: usize, field_key: &str, cond: &GateCondition, raw: &str) -> String {
    if let Some(labels) = &cond.labels {
        if let Some(label) = labels.get(index) {
            if !label.trim().is_empty() {
                return label.trim().to_string();
            }
        }
    }
    match field_key {
        "GENDER" => info::gender_label(raw),
        "COUNTRY" => info::country_label(raw),
        "OCCUPATION" => info::occupation_label(raw),
        "EDUCATION" => info::education_label(raw),
        "INTERESTS" => info::interest_label(raw),
        _ => raw.to_string(),
    }
}

fn unit_of(field_key: &str) -> &'static str {
    match field_key {
        "AGE" => "岁",
        "PROPERTY_COUNT" => "套",
        _ => "",
    }
}

fn number_text(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

fn requirement_text(field: &info::FieldDef, cond: &GateCondition) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(op) = cond.op.as_deref() {
        let text = match op {
            "in" | "not_in" => {
                let values = cond.values.as_deref().unwrap_or(&[]);
                let labels: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| value_label(i, field.key, cond, v))
                    .collect();
                let joined = labels.join(" 或 ");
                if op == "in" {
                    joined
                } else {
                    format!("不含 {joined}")
                }
            }
            "between" => {
                let min = cond.min.unwrap_or(0.0);
                let max = cond.max.unwrap_or(0.0);
                format!(
                    "{}-{}{}",
                    number_text(min),
                    number_text(max),
                    unit_of(field.key)
                )
            }
            "gte" | "lte" => {
                let bound = if field.key == "EDUCATION" {
                    cond.value.as_deref().map(info::education_label)
                } else {
                    cond.min
                        .or(cond.max)
                        .map(|v| format!("{}{}", number_text(v), unit_of(field.key)))
                };
                let symbol = if op == "gte" { "≥" } else { "≤" };
                match bound {
                    Some(b) => format!("{symbol} {b}"),
                    None => String::new(),
                }
            }
            "eq" => cond
                .value
                .as_deref()
                .map(|v| value_label(0, field.key, cond, v))
                .unwrap_or_default(),
            _ => String::new(),
        };
        if !text.is_empty() {
            parts.push(text);
        }
    }
    if let Some(level) = cond.auth_level.as_deref() {
        parts.push(format!("认证 ≥ {}", auth_level_label(level)));
    }
    match parts.len() {
        0 => String::new(),
        1 => parts.remove(0),
        _ => {
            let auth = parts.pop().unwrap_or_default();
            format!("{}（{auth}）", parts.join("，"))
        }
    }
}

// -------- 规则存储与判定 --------

#[derive(Debug, Clone, Default)]
pub struct ForumRules {
    pub browse: Option<GateRule>,
    pub operate: Option<GateRule>,
    pub legacy: Option<String>,
    /// 迁移未执行（缺列）时为 false，此时按旧 `required_auth_types` 回退。
    pub columns_available: bool,
}

impl ForumRules {
    pub fn browse_json(&self) -> Value {
        rule_value(self.browse.as_ref())
    }

    pub fn operate_json(&self) -> Value {
        rule_value(self.operate.as_ref())
    }
}

fn rule_value(rule: Option<&GateRule>) -> Value {
    rule.map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .unwrap_or(Value::Null)
}

pub async fn load_rules(pool: &MySqlPool, forum_id: i64) -> Result<ForumRules, ApiError> {
    match sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>)>(
        "SELECT browse_rule, operate_rule, required_auth_types FROM sub_forums WHERE id = ?",
    )
    .bind(forum_id)
    .fetch_optional(pool)
    .await
    {
        Ok(row) => {
            let (browse, operate, legacy) = row.unwrap_or((None, None, None));
            Ok(ForumRules {
                browse: parse_rule_text(browse.as_deref()).filter(|r| !r.is_empty()),
                operate: parse_rule_text(operate.as_deref()).filter(|r| !r.is_empty()),
                legacy,
                columns_available: true,
            })
        }
        Err(e) if db_code(&e).as_deref() == Some("1054") => {
            let legacy: Option<(Option<String>,)> =
                sqlx::query_as("SELECT required_auth_types FROM sub_forums WHERE id = ?")
                    .bind(forum_id)
                    .fetch_optional(pool)
                    .await?;
            Ok(ForumRules {
                legacy: legacy.and_then(|(v,)| v),
                columns_available: false,
                ..Default::default()
            })
        }
        Err(e) => Err(e.into()),
    }
}

/// 判定板块门槛。
pub async fn check(
    pool: &MySqlPool,
    user_id: Option<i64>,
    is_admin: bool,
    forum: &SubForum,
    kind: GateKind,
) -> Result<GateVerdict, ApiError> {
    let rules = load_rules(pool, forum.id).await?;

    if !rules.columns_available {
        // 迁移未执行：浏览不拦截，操作沿用旧 CSV 认证门槛。
        if kind == GateKind::Browse {
            return Ok(GateVerdict::allow());
        }
        let legacy = crate::auth_gate::evaluate(pool, user_id, forum.required_auth_types.as_deref())
            .await?;
        if legacy.passed {
            return Ok(GateVerdict::allow());
        }
        return Ok(GateVerdict {
            passed: false,
            login_required: user_id.is_none(),
            message: Some(legacy.message),
            missing: Vec::new(),
        });
    }

    let mut conditions: Vec<GateCondition> = Vec::new();
    if kind == GateKind::Operate {
        if let Some(browse) = &rules.browse {
            conditions.extend(browse.conditions.iter().cloned());
        }
        if let Some(operate) = &rules.operate {
            conditions.extend(operate.conditions.iter().cloned());
        }
        if rules.operate.is_none() {
            if let Some(csv) = rules.legacy.as_deref() {
                if !csv.trim().is_empty() {
                    let legacy =
                        crate::auth_gate::evaluate(pool, user_id, Some(csv)).await?;
                    if !legacy.passed {
                        return Ok(GateVerdict {
                            passed: false,
                            login_required: user_id.is_none(),
                            message: Some(legacy.message),
                            missing: Vec::new(),
                        });
                    }
                }
            }
        }
    } else if let Some(browse) = &rules.browse {
        conditions.extend(browse.conditions.iter().cloned());
    }

    if conditions.is_empty() {
        return Ok(GateVerdict::allow());
    }
    let ctx = load_context(pool, user_id, is_admin).await?;
    if ctx.exempt(forum) {
        return Ok(GateVerdict::allow());
    }
    if user_id.is_none() {
        return Ok(GateVerdict {
            passed: false,
            login_required: true,
            message: None,
            missing: Vec::new(),
        });
    }
    let missing = ctx.evaluate(&conditions);
    Ok(GateVerdict {
        passed: missing.is_empty(),
        login_required: false,
        message: None,
        missing,
    })
}

pub async fn ensure(
    pool: &MySqlPool,
    user_id: Option<i64>,
    is_admin: bool,
    forum: &SubForum,
    kind: GateKind,
) -> Result<(), ApiError> {
    let verdict = check(pool, user_id, is_admin, forum, kind).await?;
    if verdict.passed {
        Ok(())
    } else {
        Err(verdict.deny(kind, forum))
    }
}

/// 返回用户可浏览的板块 id 集合（用于列表/搜索过滤）。
pub async fn allowed_forum_ids(
    pool: &MySqlPool,
    user_id: Option<i64>,
    is_admin: bool,
) -> Result<HashSet<i64>, ApiError> {
    let rows: Vec<(i64, i64, Option<String>)> = match sqlx::query_as(
        "SELECT id, creator_id, browse_rule FROM sub_forums",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) if db_code(&e).as_deref() == Some("1054") => {
            let ids: Vec<(i64, i64)> =
                sqlx::query_as("SELECT id, creator_id FROM sub_forums")
                    .fetch_all(pool)
                    .await?;
            return Ok(ids.into_iter().map(|(id, _)| id).collect());
        }
        Err(e) => return Err(e.into()),
    };

    let ctx = load_context(pool, user_id, is_admin).await?;
    let mut allowed = HashSet::new();
    for (id, creator_id, browse_raw) in rows {
        let exempt = is_admin
            || user_id.map(|uid| uid == creator_id).unwrap_or(false)
            || ctx.manager_forums.contains(&id);
        if exempt {
            allowed.insert(id);
            continue;
        }
        let rule = parse_rule_text(browse_raw.as_deref());
        match rule {
            Some(rule) if !rule.is_empty() => {
                if user_id.is_some() && ctx.evaluate(&rule.conditions).is_empty() {
                    allowed.insert(id);
                }
            }
            _ => {
                allowed.insert(id);
            }
        }
    }
    Ok(allowed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(info_rows: Vec<(&str, Value, Option<&str>, Option<&str>)>) -> GateContext {
        let mut info = HashMap::new();
        for (key, value, level, status) in info_rows {
            info.insert(
                key.to_string(),
                InfoRow {
                    value,
                    auth_level: level.map(|s| s.to_string()),
                    auth_status: status.map(|s| s.to_string()),
                },
            );
        }
        GateContext {
            user_id: Some(1),
            is_admin: false,
            manager_forums: HashSet::new(),
            info,
        }
    }

    fn rule(json_text: &str) -> GateRule {
        let value: Value = serde_json::from_str(json_text).unwrap();
        normalize_rule(Some(&value)).unwrap().unwrap()
    }

    #[test]
    fn gender_in_with_multi_values_is_or() {
        let r = rule(r#"{"version":1,"conditions":[{"field":"GENDER","op":"in","values":["MALE","FEMALE"]}]}"#);
        let ctx = context(vec![("GENDER", json!({"gender":"MALE"}), Some("LOW"), Some("APPROVED"))]);
        assert!(ctx.evaluate(&r.conditions).is_empty());
    }

    #[test]
    fn age_between_uses_birth_date() {
        let r = rule(r#"{"version":1,"conditions":[{"field":"AGE","op":"between","min":18,"max":30}]}"#);
        let birth = chrono::Local::now().date_naive() - chrono::Duration::days(365 * 25);
        let ctx = context(vec![(
            "AGE",
            json!({"birthDate": birth.format("%Y-%m-%d").to_string()}),
            Some("LOW"),
            Some("APPROVED"),
        )]);
        assert!(ctx.evaluate(&r.conditions).is_empty());
    }

    #[test]
    fn education_gte_compares_rank() {
        let r = rule(r#"{"version":1,"conditions":[{"field":"EDUCATION","op":"gte","value":"BACHELOR"}]}"#);
        let ctx = context(vec![(
            "EDUCATION",
            json!({"education":"MASTER"}),
            Some("LOW"),
            Some("APPROVED"),
        )]);
        assert!(ctx.evaluate(&r.conditions).is_empty());
        let low = context(vec![(
            "EDUCATION",
            json!({"education":"COLLEGE"}),
            Some("LOW"),
            Some("APPROVED"),
        )]);
        assert_eq!(low.evaluate(&r.conditions)[0].reason, "VALUE_MISMATCH");
    }

    #[test]
    fn auth_level_missing_is_reported() {
        let r = rule(r#"{"version":1,"conditions":[{"field":"REAL_NAME","authLevel":"MEDIUM"}]}"#);
        let ctx = context(vec![(
            "REAL_NAME",
            json!({"fullName":"张三"}),
            Some("LOW"),
            Some("APPROVED"),
        )]);
        let missing = ctx.evaluate(&r.conditions);
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].reason, "AUTH_LEVEL_LOW");
    }

    #[test]
    fn not_filled_is_reported() {
        let r = rule(r#"{"version":1,"conditions":[{"field":"CITY","op":"in","values":["310000"]}]}"#);
        let ctx = context(vec![]);
        assert_eq!(ctx.evaluate(&r.conditions)[0].reason, "NOT_FILLED");
    }

    #[test]
    fn property_area_uses_intersection() {
        let r = rule(r#"{"version":1,"conditions":[{"field":"PROPERTY_AREA","op":"in","values":["310000","330100"]}]}"#);
        let ctx = context(vec![(
            "PROPERTY_AREA",
            json!({"cities":["330100"],"districts":[]}),
            Some("MEDIUM"),
            Some("APPROVED"),
        )]);
        assert!(ctx.evaluate(&r.conditions).is_empty());
    }

    #[test]
    fn interests_reject_auth_level() {
        let value = json!({"version":1,"conditions":[{"field":"INTERESTS","op":"in","values":["sports"],"authLevel":"LOW"}]});
        assert!(normalize_rule(Some(&value)).is_err());
    }
}
