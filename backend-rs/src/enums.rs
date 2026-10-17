use crate::error::ApiError;

/// 认证等级，对应 Java `AuthLevel`（rank + 中文标签）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthLevel {
    Low,
    Medium,
    High,
    Extreme,
}

impl AuthLevel {
    pub fn rank(self) -> i32 {
        match self {
            AuthLevel::Low => 1,
            AuthLevel::Medium => 2,
            AuthLevel::High => 3,
            AuthLevel::Extreme => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            AuthLevel::Low => "LOW",
            AuthLevel::Medium => "MEDIUM",
            AuthLevel::High => "HIGH",
            AuthLevel::Extreme => "EXTREME",
        }
    }

    /// 未知等级 → 400（对应 Java `AuthLevel.parse`）。
    pub fn parse(raw: &str) -> Result<AuthLevel, ApiError> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "LOW" => Ok(AuthLevel::Low),
            "MEDIUM" => Ok(AuthLevel::Medium),
            "HIGH" => Ok(AuthLevel::High),
            "EXTREME" => Ok(AuthLevel::Extreme),
            _ => Err(ApiError::bad_request(format!("未知认证等级: {raw}"))),
        }
    }

    pub fn parse_or_null(raw: Option<&str>) -> Option<AuthLevel> {
        raw.and_then(|s| {
            if s.trim().is_empty() {
                None
            } else {
                AuthLevel::parse(s).ok()
            }
        })
    }
}

/// 认证类型取值，对应 Java `AuthType`。
pub const AUTH_TYPES: [(&str, &str); 10] = [
    ("REAL_NAME", "实名认证"),
    ("ID_CARD", "身份证认证"),
    ("GENDER", "性别认证"),
    ("AGE", "年龄认证"),
    ("NATIONALITY", "国籍认证"),
    ("DRIVER_LICENSE", "驾照认证"),
    ("MARRIAGE_CERT", "婚姻认证"),
    ("PROPERTY_CERT", "房产认证"),
    ("ASSETS", "资产认证"),
    ("CUSTOM", "自定义认证"),
];

pub fn auth_type_label(t: &str) -> Option<&'static str> {
    AUTH_TYPES.iter().find(|(k, _)| *k == t).map(|(_, v)| *v)
}

pub fn is_known_auth_type(t: &str) -> bool {
    AUTH_TYPES.iter().any(|(k, _)| *k == t)
}

/// 动作池，对应 Java `AuthChallengeService`。
pub const CHALLENGE_ACTIONS: [&str; 5] = ["BLINK", "OPEN_MOUTH", "TURN_LEFT", "TURN_RIGHT", "NOD"];
