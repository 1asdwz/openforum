use axum::Json;
use chrono::{Duration, NaiveDateTime};
use serde::Serialize;
use serde_json::{Map, Value};

/// Convert a timestamp "read from the database" into a JSON string exactly matching Java, in two steps:
///
/// 1) `-8h` offset: the Java side's `DB_URL` sets `serverTimezone=Asia/Shanghai`, while the MySQL container actually runs in UTC;
///    Connector/J therefore interprets DATETIME as +08:00 and then outputs it in UTC, making the read time 8 hours less than the value stored in the DB.
///    To ensure both backends return identical JSON, Rust likewise subtracts 8 hours when serializing timestamps "read from the database".
///    Note: this applies only to timestamps originating from the database; timestamps computed at runtime (not via the DB) are not offset.
/// 2) String format: mirrors Java `LocalDateTime.toString()` — `yyyy-MM-ddTHH:mm:ss`,
///    plus optional fractional seconds (up to 9 digits, trailing zeros removed). chrono outputs a fixed 6/9 digits by default,
///    which would produce differences like `...48.942590` (Rust) vs `...48.94259` (Java).
pub const JAVA_TS_OFFSET_HOURS: i64 = 8;

pub fn jts(dt: NaiveDateTime) -> String {
    fmt_like_java(dt - Duration::hours(JAVA_TS_OFFSET_HOURS))
}

pub fn jts_opt(dt: Option<NaiveDateTime>) -> Option<String> {
    dt.map(jts)
}

/// Format per Java `LocalDateTime.toString()`: strip trailing zeros from fractional seconds; omit the decimal point when all zeros.
fn fmt_like_java(dt: NaiveDateTime) -> String {
    use chrono::Timelike;
    let base = dt.format("%Y-%m-%dT%H:%M:%S").to_string();
    let nanos = dt.nanosecond();
    if nanos == 0 {
        return base;
    }
    let mut frac = format!("{nanos:09}");
    while frac.ends_with('0') {
        frac.pop();
    }
    format!("{base}.{frac}")
}

/// Unified response body, mirrors Java `ApiResponse`.
pub fn ok<T: Serialize>(data: T) -> Json<Value> {
    Json(serde_json::json!({ "code": 200, "message": "ok", "data": data }))
}

pub fn ok_empty() -> Json<Value> {
    Json(serde_json::json!({ "code": 200, "message": "ok", "data": null }))
}

/// Mirrors Java `PageResponse`: content/page/size/totalElements/totalPages.
#[derive(Debug, Clone, Serialize)]
pub struct PageResponse<T> {
    pub content: Vec<T>,
    pub page: i32,
    pub size: i32,
    #[serde(rename = "totalElements")]
    pub total_elements: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
}

impl<T> PageResponse<T> {
    /// Matches Java `PageResponse.of(List, page, size, totalElements)`: totalPages = ceil(total/size).
    pub fn of(content: Vec<T>, page: i32, size: i32, total_elements: i64) -> Self {
        let total_pages = if size <= 0 {
            0
        } else {
            ((total_elements as f64) / (size as f64)).ceil() as i32
        };
        Self { content, page, size, total_elements, total_pages }
    }
}

/// In-memory pagination slice, replicating the from/to computation in Java `PostService#toPage`.
pub fn slice_page<T>(items: Vec<T>, page: i32, size: i32) -> (Vec<T>, i64) {
    let total = items.len() as i64;
    if size <= 0 {
        return (Vec::new(), total);
    }
    let size = size as usize;
    let start = ((page.max(0) as usize).saturating_mul(size)).min(items.len());
    let content = items.into_iter().skip(start).take(size).collect();
    (content, total)
}

/// Mirrors Java `JsonSupport`: parse a JSON string from the DB into an object; empty → empty object.
pub fn json_to_map(raw: Option<&str>) -> Map<String, Value> {
    match raw {
        Some(s) if !s.trim().is_empty() => match serde_json::from_str::<Value>(s) {
            Ok(Value::Object(m)) => m,
            _ => Map::new(),
        },
        _ => Map::new(),
    }
}

/// Parse a JSON string into any value; empty/invalid → Value::Null.
pub fn json_value(raw: Option<&str>) -> Value {
    match raw {
        Some(s) if !s.trim().is_empty() => {
            serde_json::from_str::<Value>(s).unwrap_or(Value::Null)
        }
        _ => Value::Null,
    }
}

/// Parse a JSON array into a list of strings; empty/invalid → empty list.
pub fn json_to_string_list(raw: Option<&str>) -> Vec<String> {
    match raw {
        Some(s) if !s.trim().is_empty() => match serde_json::from_str::<Value>(s) {
            Ok(Value::Array(arr)) => arr
                .into_iter()
                .filter_map(|v| v.as_str().map(|x| x.to_string()))
                .collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

/// Parse a JSON array into a list of longs; empty/invalid → empty list.
pub fn json_to_long_list(raw: Option<&str>) -> Vec<i64> {
    match raw {
        Some(s) if !s.trim().is_empty() => match serde_json::from_str::<Value>(s) {
            Ok(Value::Array(arr)) => arr
                .into_iter()
                .filter_map(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}
