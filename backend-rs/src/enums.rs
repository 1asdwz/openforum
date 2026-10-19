use crate::error::ApiError;

/// Auth level, mirrors Java `AuthLevel` (rank + label).
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

    /// Unknown level → 400 (mirrors Java `AuthLevel.parse`).
    pub fn parse(raw: &str) -> Result<AuthLevel, ApiError> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "LOW" => Ok(AuthLevel::Low),
            "MEDIUM" => Ok(AuthLevel::Medium),
            "HIGH" => Ok(AuthLevel::High),
            "EXTREME" => Ok(AuthLevel::Extreme),
            _ => Err(ApiError::bad_request(format!("Unknown auth level: {raw}"))),
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

/// Auth type values, mirrors Java `AuthType`.
pub const AUTH_TYPES: [(&str, &str); 13] = [
    ("REAL_NAME", "Real-name verification"),
    ("ID_CARD", "ID card verification"),
    ("GENDER", "Gender verification"),
    ("AGE", "Age verification"),
    ("NATIONALITY", "Nationality verification"),
    ("CITY", "City verification"),
    ("OCCUPATION", "Occupation verification"),
    ("EDUCATION", "Education verification"),
    ("DRIVER_LICENSE", "Driver's license verification"),
    ("MARRIAGE_CERT", "Marriage verification"),
    ("PROPERTY_CERT", "Property verification"),
    ("ASSETS", "Assets verification"),
    ("CUSTOM", "Custom verification"),
];

pub fn auth_type_label(t: &str) -> Option<&'static str> {
    AUTH_TYPES.iter().find(|(k, _)| *k == t).map(|(_, v)| *v)
}

pub fn is_known_auth_type(t: &str) -> bool {
    AUTH_TYPES.iter().any(|(k, _)| *k == t)
}

/// Action pool, mirrors Java `AuthChallengeService`.
pub const CHALLENGE_ACTIONS: [&str; 5] = ["BLINK", "OPEN_MOUTH", "TURN_LEFT", "TURN_RIGHT", "NOD"];
