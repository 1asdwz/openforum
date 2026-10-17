use chrono::{Datelike, Local, NaiveDate};
use serde_json::{json, Map, Value};

use crate::error::ApiError;

/// 信息项分组（顺序即展示顺序）。
pub const GROUPS: [(&str, &str); 5] = [
    ("IDENTITY", "身份"),
    ("LOCATION", "所在地"),
    ("CAREER", "职业与学历"),
    ("PROPERTY", "房产"),
    ("INTEREST", "兴趣"),
];

pub struct FieldDef {
    pub key: &'static str,
    pub group: &'static str,
    pub label: &'static str,
    pub value_type: &'static str,
    pub authable: bool,
    pub gate_ops: &'static str,
    pub sort: i32,
    pub enabled: bool,
    pub options: &'static str,
}

const OPTIONS_GENDER: &str = r#"[{"value":"MALE","label":"男"},{"value":"FEMALE","label":"女"}]"#;

const OPTIONS_COUNTRY: &str = r#"[
{"value":"CN","label":"中国"},{"value":"HK","label":"中国香港"},{"value":"TW","label":"中国台湾"},
{"value":"MO","label":"中国澳门"},{"value":"US","label":"美国"},{"value":"JP","label":"日本"},
{"value":"KR","label":"韩国"},{"value":"SG","label":"新加坡"},{"value":"GB","label":"英国"},
{"value":"FR","label":"法国"},{"value":"DE","label":"德国"},{"value":"CA","label":"加拿大"},
{"value":"AU","label":"澳大利亚"},{"value":"NZ","label":"新西兰"},{"value":"RU","label":"俄罗斯"},
{"value":"IN","label":"印度"},{"value":"TH","label":"泰国"},{"value":"MY","label":"马来西亚"},
{"value":"ID","label":"印度尼西亚"},{"value":"PH","label":"菲律宾"},{"value":"VN","label":"越南"},
{"value":"OTHER","label":"其他"}
]"#;

const OPTIONS_OCCUPATION: &str = r#"[
{"value":"INTERNET","label":"互联网/科技"},{"value":"FINANCE","label":"金融"},
{"value":"EDUCATION","label":"教育"},{"value":"MEDICAL","label":"医疗健康"},
{"value":"GOVERNMENT","label":"政府/公共事业"},{"value":"MANUFACTURING","label":"制造业"},
{"value":"SERVICE","label":"服务业"},{"value":"OTHER","label":"其他"}
]"#;

const OPTIONS_EDUCATION: &str = r#"[
{"value":"HIGH_SCHOOL","label":"高中及以下","order":1},
{"value":"COLLEGE","label":"大专","order":2},
{"value":"BACHELOR","label":"本科","order":3},
{"value":"MASTER","label":"硕士","order":4},
{"value":"DOCTOR","label":"博士","order":5}
]"#;

const OPTIONS_INTERESTS: &str = r#"[
{"value":"sports","label":"运动"},{"value":"reading","label":"读书"},{"value":"travel","label":"旅行"},
{"value":"music","label":"音乐"},{"value":"movie","label":"电影"},{"value":"food","label":"美食"},
{"value":"photography","label":"摄影"},{"value":"gaming","label":"游戏"},{"value":"coding","label":"编程"},
{"value":"fitness","label":"健身"},{"value":"coffee","label":"咖啡"},{"value":"pets","label":"宠物"},
{"value":"gardening","label":"园艺"},{"value":"painting","label":"绘画"},{"value":"handcraft","label":"手工"}
]"#;

pub const VISIBLE_FIELDS: [FieldDef; 10] = [
    FieldDef {
        key: "REAL_NAME",
        group: "IDENTITY",
        label: "实名",
        value_type: "text",
        authable: true,
        gate_ops: "",
        sort: 10,
        enabled: true,
        options: "[]",
    },
    FieldDef {
        key: "GENDER",
        group: "IDENTITY",
        label: "性别",
        value_type: "enum",
        authable: true,
        gate_ops: "in,not_in",
        sort: 20,
        enabled: true,
        options: OPTIONS_GENDER,
    },
    FieldDef {
        key: "AGE",
        group: "IDENTITY",
        label: "年龄",
        value_type: "date",
        authable: true,
        gate_ops: "between,gte,lte",
        sort: 30,
        enabled: true,
        options: "[]",
    },
    FieldDef {
        key: "COUNTRY",
        group: "LOCATION",
        label: "国家",
        value_type: "enum",
        authable: true,
        gate_ops: "in,not_in",
        sort: 40,
        enabled: true,
        options: OPTIONS_COUNTRY,
    },
    FieldDef {
        key: "CITY",
        group: "LOCATION",
        label: "城市",
        value_type: "region",
        authable: true,
        gate_ops: "in,not_in",
        sort: 50,
        enabled: true,
        options: "[]",
    },
    FieldDef {
        key: "OCCUPATION",
        group: "CAREER",
        label: "职业",
        value_type: "enum",
        authable: true,
        gate_ops: "in,not_in",
        sort: 60,
        enabled: true,
        options: OPTIONS_OCCUPATION,
    },
    FieldDef {
        key: "EDUCATION",
        group: "CAREER",
        label: "学历",
        value_type: "ordinal",
        authable: true,
        gate_ops: "gte,lte,eq",
        sort: 70,
        enabled: true,
        options: OPTIONS_EDUCATION,
    },
    FieldDef {
        key: "PROPERTY_AREA",
        group: "PROPERTY",
        label: "房产区域",
        value_type: "region_multi",
        authable: true,
        gate_ops: "in,not_in",
        sort: 80,
        enabled: true,
        options: "[]",
    },
    FieldDef {
        key: "PROPERTY_COUNT",
        group: "PROPERTY",
        label: "房产数量",
        value_type: "int",
        authable: true,
        gate_ops: "between,gte,lte",
        sort: 90,
        enabled: true,
        options: "[]",
    },
    FieldDef {
        key: "INTERESTS",
        group: "INTEREST",
        label: "兴趣",
        value_type: "tags",
        authable: false,
        gate_ops: "in,not_in",
        sort: 100,
        enabled: true,
        options: OPTIONS_INTERESTS,
    },
];

/// 旧认证类型对应的隐藏字段，仅用于旧规则兼容。
pub const HIDDEN_FIELDS: [FieldDef; 6] = [
    hidden("ID_CARD", "身份证", 110),
    hidden("PROPERTY_CERT", "房产证明", 120),
    hidden("ASSETS", "资产证明", 130),
    hidden("MARRIAGE_CERT", "婚姻认证", 140),
    hidden("DRIVER_LICENSE", "驾驶证", 150),
    hidden("CUSTOM", "自定义认证", 160),
];

const fn hidden(key: &'static str, label: &'static str, sort: i32) -> FieldDef {
    FieldDef {
        key,
        group: "IDENTITY",
        label,
        value_type: "text",
        authable: true,
        gate_ops: "",
        sort,
        enabled: false,
        options: "[]",
    }
}

pub fn all_fields() -> impl Iterator<Item = &'static FieldDef> {
    VISIBLE_FIELDS.iter().chain(HIDDEN_FIELDS.iter())
}

pub fn find_field(key: &str) -> Option<&'static FieldDef> {
    let key = key.trim().to_ascii_uppercase();
    all_fields().find(|f| f.key == key)
}

pub fn is_visible(key: &str) -> bool {
    VISIBLE_FIELDS.iter().any(|f| f.key == key)
}

pub fn options_value(field: &FieldDef) -> Value {
    serde_json::from_str(field.options).unwrap_or(Value::Array(Vec::new()))
}

pub fn gate_ops(field: &FieldDef) -> Vec<&'static str> {
    field
        .gate_ops
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// 认证等级目录（信息项可选的认证等级）。
pub fn auth_levels(field: &FieldDef) -> Vec<&'static str> {
    if field.authable {
        vec!["LOW", "MEDIUM", "HIGH", "EXTREME"]
    } else {
        Vec::new()
    }
}

// -------- 值校验 --------

fn as_object(value: &Value) -> Result<&Map<String, Value>, ApiError> {
    value
        .as_object()
        .ok_or_else(|| ApiError::bad_request("value 必须是 JSON 对象"))
}

fn text_of(map: &Map<String, Value>, key: &str) -> String {
    match map.get(key) {
        Some(Value::String(s)) => s.trim().to_string(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

fn string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.trim().to_string()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
            .filter(|s| !s.is_empty())
            .collect(),
        Some(Value::String(s)) => s
            .split([',', '，', ' '])
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn dedup(items: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in items {
        if !out.contains(&item) {
            out.push(item);
        }
    }
    out
}

pub fn normalize_gender(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_uppercase().as_str() {
        "MALE" | "M" | "男" | "男性" => Some("MALE"),
        "FEMALE" | "F" | "女" | "女性" => Some("FEMALE"),
        _ => None,
    }
}

pub fn gender_label(raw: &str) -> String {
    match normalize_gender(raw) {
        Some("MALE") => "男".to_string(),
        Some("FEMALE") => "女".to_string(),
        _ => raw.to_string(),
    }
}

pub fn country_code(raw: &str) -> Option<String> {
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }
    let upper = text.to_ascii_uppercase();
    if upper.chars().count() == 2 && upper.chars().all(|c| c.is_ascii_uppercase()) {
        return Some(upper);
    }
    let code = match text {
        "中国" | "中华人民共和国" | "中国大陆" => "CN",
        "中国香港" | "香港" => "HK",
        "中国台湾" | "台湾" => "TW",
        "中国澳门" | "澳门" => "MO",
        "美国" => "US",
        "日本" => "JP",
        "韩国" | "南韩" => "KR",
        "新加坡" => "SG",
        "英国" | "大不列颠" => "GB",
        "法国" => "FR",
        "德国" => "DE",
        "加拿大" => "CA",
        "澳大利亚" | "澳洲" => "AU",
        "新西兰" => "NZ",
        "俄罗斯" => "RU",
        "印度" => "IN",
        "泰国" => "TH",
        "马来西亚" => "MY",
        "印度尼西亚" | "印尼" => "ID",
        "菲律宾" => "PH",
        "越南" => "VN",
        _ => return None,
    };
    Some(code.to_string())
}

pub fn country_label(code: &str) -> String {
    let options = serde_json::from_str::<Value>(OPTIONS_COUNTRY).unwrap_or(Value::Null);
    if let Some(items) = options.as_array() {
        for item in items {
            if item.get("value").and_then(|v| v.as_str()) == Some(code) {
                if let Some(label) = item.get("label").and_then(|v| v.as_str()) {
                    return label.to_string();
                }
            }
        }
    }
    code.to_string()
}

pub fn education_rank(raw: &str) -> Option<i32> {
    match raw.trim().to_ascii_uppercase().as_str() {
        "HIGH_SCHOOL" => Some(1),
        "COLLEGE" => Some(2),
        "BACHELOR" => Some(3),
        "MASTER" => Some(4),
        "DOCTOR" => Some(5),
        _ => None,
    }
}

pub fn education_label(raw: &str) -> String {
    match education_rank(raw) {
        Some(1) => "高中及以下".to_string(),
        Some(2) => "大专".to_string(),
        Some(3) => "本科".to_string(),
        Some(4) => "硕士".to_string(),
        Some(5) => "博士".to_string(),
        _ => raw.to_string(),
    }
}

pub fn occupation_label(raw: &str) -> String {
    match raw.trim().to_ascii_uppercase().as_str() {
        "INTERNET" => "互联网/科技".to_string(),
        "FINANCE" => "金融".to_string(),
        "EDUCATION" => "教育".to_string(),
        "MEDICAL" => "医疗健康".to_string(),
        "GOVERNMENT" => "政府/公共事业".to_string(),
        "MANUFACTURING" => "制造业".to_string(),
        "SERVICE" => "服务业".to_string(),
        "OTHER" => "其他".to_string(),
        other => other.to_string(),
    }
}

pub fn is_valid_occupation(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_uppercase().as_str(),
        "INTERNET"
            | "FINANCE"
            | "EDUCATION"
            | "MEDICAL"
            | "GOVERNMENT"
            | "MANUFACTURING"
            | "SERVICE"
            | "OTHER"
    )
}

pub fn interest_label(raw: &str) -> String {
    let options = serde_json::from_str::<Value>(OPTIONS_INTERESTS).unwrap_or(Value::Null);
    if let Some(items) = options.as_array() {
        for item in items {
            if item.get("value").and_then(|v| v.as_str()) == Some(raw) {
                if let Some(label) = item.get("label").and_then(|v| v.as_str()) {
                    return label.to_string();
                }
            }
        }
    }
    raw.to_string()
}

/// 校验并规范化用户信息值。
pub fn validate_value(field_key: &str, value: &Value) -> Result<Value, ApiError> {
    let field = find_field(field_key)
        .ok_or_else(|| ApiError::bad_request(format!("未知信息项: {field_key}")))?;
    let map = as_object(value)?;
    match field.key {
        "REAL_NAME" => {
            let full_name = text_of(map, "fullName");
            let len = full_name.chars().count();
            if !(2..=50).contains(&len) {
                return Err(ApiError::bad_request("真实姓名长度需在 2-50 之间"));
            }
            Ok(json!({ "fullName": full_name }))
        }
        "GENDER" => {
            let raw = text_of(map, "gender");
            let gender = normalize_gender(&raw)
                .ok_or_else(|| ApiError::bad_request("性别取值不合法"))?;
            Ok(json!({ "gender": gender }))
        }
        "AGE" => {
            let birth = text_of(map, "birthDate");
            if !birth.is_empty() {
                let date = NaiveDate::parse_from_str(&birth, "%Y-%m-%d")
                    .map_err(|_| ApiError::bad_request("出生日期格式应为 YYYY-MM-DD"))?;
                let age = age_from_date(date);
                if !(0..=120).contains(&age) {
                    return Err(ApiError::bad_request("年龄需在 0-120 之间"));
                }
                return Ok(json!({ "birthDate": date.format("%Y-%m-%d").to_string() }));
            }
            let raw_age = map
                .get("age")
                .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())))
                .ok_or_else(|| ApiError::bad_request("请填写出生日期"))?;
            if !(0..=120).contains(&raw_age) {
                return Err(ApiError::bad_request("年龄需在 0-120 之间"));
            }
            Ok(json!({ "age": raw_age }))
        }
        "COUNTRY" => {
            let raw = text_of(map, "country");
            let code = country_code(&raw)
                .ok_or_else(|| ApiError::bad_request("国家取值不合法"))?;
            Ok(json!({ "country": code }))
        }
        "CITY" => {
            let city = text_of(map, "city");
            if city.is_empty() || city.chars().count() > 40 {
                return Err(ApiError::bad_request("请选择城市"));
            }
            let country = country_code(&text_of(map, "country")).unwrap_or_else(|| "CN".to_string());
            Ok(json!({ "country": country, "city": city }))
        }
        "OCCUPATION" => {
            let category = text_of(map, "category").to_ascii_uppercase();
            if !is_valid_occupation(&category) {
                return Err(ApiError::bad_request("职业分类不合法"));
            }
            let title = text_of(map, "title");
            if title.chars().count() > 50 {
                return Err(ApiError::bad_request("职位名称不能超过 50 字"));
            }
            Ok(json!({ "category": category, "title": title }))
        }
        "EDUCATION" => {
            let raw = text_of(map, "education").to_ascii_uppercase();
            if education_rank(&raw).is_none() {
                return Err(ApiError::bad_request("学历取值不合法"));
            }
            Ok(json!({ "education": raw }))
        }
        "PROPERTY_AREA" => {
            let cities = dedup(string_list(map.get("cities")));
            let districts = dedup(string_list(map.get("districts")));
            if cities.is_empty() && districts.is_empty() {
                return Err(ApiError::bad_request("请至少选择一个房产区域"));
            }
            Ok(json!({ "cities": cities, "districts": districts }))
        }
        "PROPERTY_COUNT" => {
            let count = map
                .get("count")
                .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())))
                .ok_or_else(|| ApiError::bad_request("请填写房产数量"))?;
            if !(0..=100).contains(&count) {
                return Err(ApiError::bad_request("房产数量需在 0-100 之间"));
            }
            Ok(json!({ "count": count }))
        }
        "INTERESTS" => {
            let tags = dedup(string_list(map.get("tags")));
            if tags.len() > 30 {
                return Err(ApiError::bad_request("兴趣标签最多 30 个"));
            }
            if tags.iter().any(|t| t.chars().count() > 30) {
                return Err(ApiError::bad_request("单个兴趣标签不能超过 30 字"));
            }
            Ok(json!({ "tags": tags }))
        }
        _ => Ok(Value::Object(map.clone())),
    }
}

pub fn value_text(field_key: &str, value: &Value) -> String {
    let map = match value.as_object() {
        Some(m) => m,
        None => return String::new(),
    };
    match field_key {
        "REAL_NAME" => text_of(map, "fullName"),
        "GENDER" => gender_label(&text_of(map, "gender")),
        "AGE" => match age_of(value) {
            Some(age) => {
                let birth = text_of(map, "birthDate");
                if birth.is_empty() {
                    format!("{age} 岁")
                } else {
                    format!("{age} 岁（{birth}）")
                }
            }
            None => String::new(),
        },
        "COUNTRY" => country_label(&text_of(map, "country")),
        "CITY" => {
            let city = text_of(map, "city");
            if city.is_empty() {
                String::new()
            } else if city.chars().all(|c| c.is_ascii_digit()) {
                city
            } else {
                city
            }
        }
        "OCCUPATION" => {
            let category = occupation_label(&text_of(map, "category"));
            let title = text_of(map, "title");
            if title.is_empty() {
                category
            } else {
                format!("{category} · {title}")
            }
        }
        "EDUCATION" => education_label(&text_of(map, "education")),
        "PROPERTY_AREA" => {
            let mut parts = string_list(map.get("cities"));
            parts.extend(string_list(map.get("districts")));
            parts.join("、")
        }
        "PROPERTY_COUNT" => map
            .get("count")
            .and_then(|v| v.as_i64())
            .map(|n| format!("{n} 套"))
            .unwrap_or_default(),
        "INTERESTS" => {
            let tags = string_list(map.get("tags"));
            tags.iter().map(|t| interest_label(t)).collect::<Vec<_>>().join("、")
        }
        _ => {
            if map.is_empty() {
                "已认证".to_string()
            } else {
                "已认证".to_string()
            }
        }
    }
}

/// 是否已填写（空对象/空数组/空字符串视为未填写）。
pub fn is_filled(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(s) => !s.trim().is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => map.values().any(is_filled),
        _ => true,
    }
}

pub fn age_from_date(birth: NaiveDate) -> i32 {
    let today = Local::now().date_naive();
    let mut age = today.year() - birth.year();
    if (today.month(), today.day()) < (birth.month(), birth.day()) {
        age -= 1;
    }
    age
}

/// 从值中取年龄：优先出生日期，兼容旧数据的直接年龄。
pub fn age_of(value: &Value) -> Option<i32> {
    let map = value.as_object()?;
    if let Some(Value::String(birth)) = map.get("birthDate") {
        if let Ok(date) = NaiveDate::parse_from_str(birth.trim(), "%Y-%m-%d") {
            return Some(age_from_date(date));
        }
    }
    map.get("age").and_then(|v| {
        v.as_i64()
            .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
            .map(|n| n as i32)
    })
}

// -------- 认证类型 → 信息项映射 --------

fn push_if_non_empty(out: &mut Vec<(String, Value)>, key: &str, value: Value) {
    if is_filled(&value) {
        out.push((key.to_string(), value));
    }
}

/// 返回某次认证通过后应授予的信息项值。
pub fn granted_fields(auth_type: &str, data: &Map<String, Value>) -> Vec<(String, Value)> {
    let mut out: Vec<(String, Value)> = Vec::new();
    let full_name = {
        let a = text_of(data, "fullName");
        if a.is_empty() {
            text_of(data, "name")
        } else {
            a
        }
    };
    match auth_type {
        "REAL_NAME" => {
            push_if_non_empty(&mut out, "REAL_NAME", json!({ "fullName": full_name }));
        }
        "GENDER" => {
            if let Some(g) = normalize_gender(&text_of(data, "gender")) {
                out.push(("GENDER".to_string(), json!({ "gender": g })));
            }
        }
        "AGE" => {
            let birth = text_of(data, "birthDate");
            if !birth.is_empty() {
                if let Ok(d) = NaiveDate::parse_from_str(&birth, "%Y-%m-%d") {
                    out.push((
                        "AGE".to_string(),
                        json!({ "birthDate": d.format("%Y-%m-%d").to_string() }),
                    ));
                }
            } else if let Some(age) = data.get("age").and_then(|v| v.as_i64()) {
                out.push(("AGE".to_string(), json!({ "age": age })));
            }
        }
        "NATIONALITY" => {
            if let Some(code) = country_code(&text_of(data, "nationality")) {
                out.push(("COUNTRY".to_string(), json!({ "country": code })));
            }
        }
        "ID_CARD" => {
            out.push(("ID_CARD".to_string(), json!({ "verified": true })));
            push_if_non_empty(&mut out, "REAL_NAME", json!({ "fullName": full_name }));
            if let Some(g) = normalize_gender(&text_of(data, "gender")) {
                out.push(("GENDER".to_string(), json!({ "gender": g })));
            }
            let birth = text_of(data, "birthDate");
            if let Ok(d) = NaiveDate::parse_from_str(&birth, "%Y-%m-%d") {
                out.push((
                    "AGE".to_string(),
                    json!({ "birthDate": d.format("%Y-%m-%d").to_string() }),
                ));
            }
            if let Some(code) = country_code(&text_of(data, "country")) {
                out.push(("COUNTRY".to_string(), json!({ "country": code })));
            }
            let city = text_of(data, "city");
            if !city.is_empty() {
                let country = country_code(&text_of(data, "country"))
                    .unwrap_or_else(|| "CN".to_string());
                out.push(("CITY".to_string(), json!({ "country": country, "city": city })));
            }
        }
        "PROPERTY_CERT" => {
            out.push(("PROPERTY_CERT".to_string(), json!({ "verified": true })));
            let cities = dedup(string_list(data.get("cities")));
            let districts = dedup(string_list(data.get("districts")));
            if !cities.is_empty() || !districts.is_empty() {
                out.push((
                    "PROPERTY_AREA".to_string(),
                    json!({ "cities": cities, "districts": districts }),
                ));
            }
            if let Some(count) = data
                .get("count")
                .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())))
            {
                out.push(("PROPERTY_COUNT".to_string(), json!({ "count": count })));
            }
        }
        "EDUCATION" => {
            let raw = text_of(data, "education").to_ascii_uppercase();
            if education_rank(&raw).is_some() {
                out.push(("EDUCATION".to_string(), json!({ "education": raw })));
            }
        }
        "OCCUPATION" => {
            let category = text_of(data, "category").to_ascii_uppercase();
            if is_valid_occupation(&category) {
                out.push((
                    "OCCUPATION".to_string(),
                    json!({ "category": category, "title": text_of(data, "title") }),
                ));
            }
        }
        "CITY" => {
            let city = text_of(data, "city");
            if !city.is_empty() {
                let country = country_code(&text_of(data, "country"))
                    .unwrap_or_else(|| "CN".to_string());
                out.push(("CITY".to_string(), json!({ "country": country, "city": city })));
            }
        }
        "ASSETS" | "MARRIAGE_CERT" | "DRIVER_LICENSE" | "CUSTOM" => {
            out.push((auth_type.to_string(), json!({ "verified": true })));
        }
        _ => {}
    }
    out
}

/// 审核通过后写入/更新用户信息项（与审核事务共用连接）。
pub async fn grant_user_info(
    conn: &mut sqlx::MySqlConnection,
    user_id: i64,
    auth_type: &str,
    auth_id: i64,
    level: &str,
    data: &Map<String, Value>,
) -> Result<(), ApiError> {
    for (field_key, value) in granted_fields(auth_type, data) {
        sqlx::query(
            "INSERT INTO user_info (user_id, field_key, value_json, auth_level, auth_status, auth_id, created_at, updated_at) \
             VALUES (?, ?, ?, ?, 'APPROVED', ?, NOW(), NOW()) \
             ON DUPLICATE KEY UPDATE value_json=VALUES(value_json), auth_level=VALUES(auth_level), \
             auth_status='APPROVED', auth_id=VALUES(auth_id), updated_at=NOW()",
        )
        .bind(user_id)
        .bind(&field_key)
        .bind(value.to_string())
        .bind(level)
        .bind(auth_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}
