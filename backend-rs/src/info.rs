use chrono::{Datelike, Local, NaiveDate};
use serde_json::{json, Map, Value};

use crate::error::ApiError;

/// Info field groups (order is display order).
pub const GROUPS: [(&str, &str); 5] = [
    ("IDENTITY", "Identity"),
    ("LOCATION", "Location"),
    ("CAREER", "Career & education"),
    ("PROPERTY", "Property"),
    ("INTEREST", "Interests"),
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

const OPTIONS_GENDER: &str = r#"[{"value":"MALE","label":"Male"},{"value":"FEMALE","label":"Female"}]"#;

const OPTIONS_COUNTRY: &str = r#"[
{"value":"CN","label":"China"},{"value":"HK","label":"Hong Kong"},{"value":"TW","label":"Taiwan"},
{"value":"MO","label":"Macao"},{"value":"US","label":"United States"},{"value":"JP","label":"Japan"},
{"value":"KR","label":"South Korea"},{"value":"SG","label":"Singapore"},{"value":"GB","label":"United Kingdom"},
{"value":"FR","label":"France"},{"value":"DE","label":"Germany"},{"value":"CA","label":"Canada"},
{"value":"AU","label":"Australia"},{"value":"NZ","label":"New Zealand"},{"value":"RU","label":"Russia"},
{"value":"IN","label":"India"},{"value":"TH","label":"Thailand"},{"value":"MY","label":"Malaysia"},
{"value":"ID","label":"Indonesia"},{"value":"PH","label":"Philippines"},{"value":"VN","label":"Vietnam"},
{"value":"OTHER","label":"Other"}
]"#;

const OPTIONS_OCCUPATION: &str = r#"[
{"value":"INTERNET","label":"Internet/Tech"},{"value":"FINANCE","label":"Finance"},
{"value":"EDUCATION","label":"Education"},{"value":"MEDICAL","label":"Healthcare"},
{"value":"GOVERNMENT","label":"Government/Public sector"},{"value":"MANUFACTURING","label":"Manufacturing"},
{"value":"SERVICE","label":"Services"},{"value":"OTHER","label":"Other"}
]"#;

const OPTIONS_EDUCATION: &str = r#"[
{"value":"HIGH_SCHOOL","label":"High school or below","order":1},
{"value":"COLLEGE","label":"Associate degree","order":2},
{"value":"BACHELOR","label":"Bachelor's degree","order":3},
{"value":"MASTER","label":"Master's degree","order":4},
{"value":"DOCTOR","label":"Doctorate","order":5}
]"#;

const OPTIONS_INTERESTS: &str = r#"[
{"value":"sports","label":"Sports"},{"value":"reading","label":"Reading"},{"value":"travel","label":"Travel"},
{"value":"music","label":"Music"},{"value":"movie","label":"Movies"},{"value":"food","label":"Food"},
{"value":"photography","label":"Photography"},{"value":"gaming","label":"Gaming"},{"value":"coding","label":"Coding"},
{"value":"fitness","label":"Fitness"},{"value":"coffee","label":"Coffee"},{"value":"pets","label":"Pets"},
{"value":"gardening","label":"Gardening"},{"value":"painting","label":"Painting"},{"value":"handcraft","label":"Handcraft"}
]"#;

pub const VISIBLE_FIELDS: [FieldDef; 10] = [
    FieldDef {
        key: "REAL_NAME",
        group: "IDENTITY",
        label: "Real name",
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
        label: "Gender",
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
        label: "Age",
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
        label: "Country",
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
        label: "City",
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
        label: "Occupation",
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
        label: "Education",
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
        label: "Property area",
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
        label: "Property count",
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
        label: "Interests",
        value_type: "tags",
        authable: false,
        gate_ops: "in,not_in",
        sort: 100,
        enabled: true,
        options: OPTIONS_INTERESTS,
    },
];

/// Hidden fields corresponding to legacy auth types, kept only for backward-compatible rules.
pub const HIDDEN_FIELDS: [FieldDef; 6] = [
    hidden("ID_CARD", "ID card", 110),
    hidden("PROPERTY_CERT", "Property certificate", 120),
    hidden("ASSETS", "Asset certificate", 130),
    hidden("MARRIAGE_CERT", "Marriage certificate", 140),
    hidden("DRIVER_LICENSE", "Driver's license", 150),
    hidden("CUSTOM", "Custom verification", 160),
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

/// Auth level catalog (the auth levels selectable for an info field).
pub fn auth_levels(field: &FieldDef) -> Vec<&'static str> {
    if field.authable {
        vec!["LOW", "MEDIUM", "HIGH", "EXTREME"]
    } else {
        Vec::new()
    }
}

// -------- Value validation --------

fn as_object(value: &Value) -> Result<&Map<String, Value>, ApiError> {
    value
        .as_object()
        .ok_or_else(|| ApiError::bad_request("value must be a JSON object"))
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
        "MALE" | "M" | "MAN" => Some("MALE"),
        "FEMALE" | "F" | "WOMAN" => Some("FEMALE"),
        _ => None,
    }
}

pub fn gender_label(raw: &str) -> String {
    match normalize_gender(raw) {
        Some("MALE") => "Male".to_string(),
        Some("FEMALE") => "Female".to_string(),
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
        "China" | "People's Republic of China" | "Mainland China" => "CN",
        "Hong Kong" | "Hong Kong SAR" => "HK",
        "Taiwan" | "Taiwan region" => "TW",
        "Macao" | "Macau" | "Macao SAR" => "MO",
        "United States" | "USA" => "US",
        "Japan" => "JP",
        "South Korea" | "Korea" => "KR",
        "Singapore" => "SG",
        "United Kingdom" | "Great Britain" => "GB",
        "France" => "FR",
        "Germany" => "DE",
        "Canada" => "CA",
        "Australia" => "AU",
        "New Zealand" => "NZ",
        "Russia" => "RU",
        "India" => "IN",
        "Thailand" => "TH",
        "Malaysia" => "MY",
        "Indonesia" => "ID",
        "Philippines" => "PH",
        "Vietnam" => "VN",
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
        Some(1) => "High school or below".to_string(),
        Some(2) => "Associate degree".to_string(),
        Some(3) => "Bachelor's degree".to_string(),
        Some(4) => "Master's degree".to_string(),
        Some(5) => "Doctorate".to_string(),
        _ => raw.to_string(),
    }
}

pub fn occupation_label(raw: &str) -> String {
    match raw.trim().to_ascii_uppercase().as_str() {
        "INTERNET" => "Internet/Tech".to_string(),
        "FINANCE" => "Finance".to_string(),
        "EDUCATION" => "Education".to_string(),
        "MEDICAL" => "Healthcare".to_string(),
        "GOVERNMENT" => "Government/Public sector".to_string(),
        "MANUFACTURING" => "Manufacturing".to_string(),
        "SERVICE" => "Services".to_string(),
        "OTHER" => "Other".to_string(),
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

/// Validate and normalize a user info value.
pub fn validate_value(field_key: &str, value: &Value) -> Result<Value, ApiError> {
    let field = find_field(field_key)
        .ok_or_else(|| ApiError::bad_request(format!("Unknown info field: {field_key}")))?;
    let map = as_object(value)?;
    match field.key {
        "REAL_NAME" => {
            let full_name = text_of(map, "fullName");
            let len = full_name.chars().count();
            if !(2..=50).contains(&len) {
                return Err(ApiError::bad_request("Real name must be 2-50 characters"));
            }
            Ok(json!({ "fullName": full_name }))
        }
        "GENDER" => {
            let raw = text_of(map, "gender");
            let gender = normalize_gender(&raw)
                .ok_or_else(|| ApiError::bad_request("Invalid gender value"))?;
            Ok(json!({ "gender": gender }))
        }
        "AGE" => {
            let birth = text_of(map, "birthDate");
            if !birth.is_empty() {
                let date = NaiveDate::parse_from_str(&birth, "%Y-%m-%d")
                    .map_err(|_| ApiError::bad_request("Birth date must use YYYY-MM-DD format"))?;
                let age = age_from_date(date);
                if !(0..=120).contains(&age) {
                    return Err(ApiError::bad_request("Age must be between 0 and 120"));
                }
                return Ok(json!({ "birthDate": date.format("%Y-%m-%d").to_string() }));
            }
            let raw_age = map
                .get("age")
                .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())))
                .ok_or_else(|| ApiError::bad_request("Please provide a birth date"))?;
            if !(0..=120).contains(&raw_age) {
                return Err(ApiError::bad_request("Age must be between 0 and 120"));
            }
            Ok(json!({ "age": raw_age }))
        }
        "COUNTRY" => {
            let raw = text_of(map, "country");
            let code = country_code(&raw)
                .ok_or_else(|| ApiError::bad_request("Invalid country value"))?;
            Ok(json!({ "country": code }))
        }
        "CITY" => {
            let city = text_of(map, "city");
            if city.is_empty() || city.chars().count() > 40 {
                return Err(ApiError::bad_request("Please select a city"));
            }
            let country = country_code(&text_of(map, "country")).unwrap_or_else(|| "CN".to_string());
            Ok(json!({ "country": country, "city": city }))
        }
        "OCCUPATION" => {
            let category = text_of(map, "category").to_ascii_uppercase();
            if !is_valid_occupation(&category) {
                return Err(ApiError::bad_request("Invalid occupation category"));
            }
            let title = text_of(map, "title");
            if title.chars().count() > 50 {
                return Err(ApiError::bad_request("Job title cannot exceed 50 characters"));
            }
            Ok(json!({ "category": category, "title": title }))
        }
        "EDUCATION" => {
            let raw = text_of(map, "education").to_ascii_uppercase();
            if education_rank(&raw).is_none() {
                return Err(ApiError::bad_request("Invalid education value"));
            }
            Ok(json!({ "education": raw }))
        }
        "PROPERTY_AREA" => {
            let cities = dedup(string_list(map.get("cities")));
            let districts = dedup(string_list(map.get("districts")));
            if cities.is_empty() && districts.is_empty() {
                return Err(ApiError::bad_request("Select at least one property area"));
            }
            Ok(json!({ "cities": cities, "districts": districts }))
        }
        "PROPERTY_COUNT" => {
            let count = map
                .get("count")
                .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())))
                .ok_or_else(|| ApiError::bad_request("Please provide the property count"))?;
            if !(0..=100).contains(&count) {
                return Err(ApiError::bad_request("Property count must be between 0 and 100"));
            }
            Ok(json!({ "count": count }))
        }
        "INTERESTS" => {
            let tags = dedup(string_list(map.get("tags")));
            if tags.len() > 30 {
                return Err(ApiError::bad_request("At most 30 interest tags"));
            }
            if tags.iter().any(|t| t.chars().count() > 30) {
                return Err(ApiError::bad_request("Each interest tag cannot exceed 30 characters"));
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
                    format!("{age} years old")
                } else {
                    format!("{age} years old ({birth})")
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
            parts.join(", ")
        }
        "PROPERTY_COUNT" => map
            .get("count")
            .and_then(|v| v.as_i64())
            .map(|n| format!("{n} properties"))
            .unwrap_or_default(),
        "INTERESTS" => {
            let tags = string_list(map.get("tags"));
            tags.iter().map(|t| interest_label(t)).collect::<Vec<_>>().join(", ")
        }
        _ => {
            if map.is_empty() {
                "Verified".to_string()
            } else {
                "Verified".to_string()
            }
        }
    }
}

/// Whether a value is filled in (empty objects/arrays/strings count as not filled).
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

/// Extract age from a value: prefer birth date, but support legacy direct-age data.
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

// -------- Auth type → info field mapping --------

fn push_if_non_empty(out: &mut Vec<(String, Value)>, key: &str, value: Value) {
    if is_filled(&value) {
        out.push((key.to_string(), value));
    }
}

/// Returns the info field values that should be granted after an auth passes.
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

/// Write/update the user's info fields after approval (shares the review transaction's connection).
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
