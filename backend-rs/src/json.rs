use axum::Json;
use chrono::{Duration, NaiveDateTime};
use serde::Serialize;
use serde_json::{Map, Value};

/// 将「数据库读出的」时间戳转换为与 Java 完全一致的 JSON 字符串，两层处理：
///
/// 1) `-8h` 偏移：Java 端 `DB_URL` 设置了 `serverTimezone=Asia/Shanghai`，而 MySQL 容器实际运行在 UTC；
///    Connector/J 因此把 DATETIME 按 +08:00 解释后再以 UTC 输出，读出的时间比库中存储值少 8 小时。
///    为保证两个后端返回完全一致的 JSON，Rust 在序列化「数据库读出的」时间戳时同样减 8 小时。
///    注意：仅适用于来自数据库的时间戳；由代码即时计算（未经过 DB）的时间戳不偏移。
/// 2) 字符串格式：对齐 Java `LocalDateTime.toString()` —— `yyyy-MM-ddTHH:mm:ss`，
///    外加可选的小数秒（最多 9 位、去掉末尾的 0）。chrono 默认固定输出 6/9 位小数，
///    会产生 `...48.942590`（Rust）与 `...48.94259`（Java）的差异。
pub const JAVA_TS_OFFSET_HOURS: i64 = 8;

pub fn jts(dt: NaiveDateTime) -> String {
    fmt_like_java(dt - Duration::hours(JAVA_TS_OFFSET_HOURS))
}

pub fn jts_opt(dt: Option<NaiveDateTime>) -> Option<String> {
    dt.map(jts)
}

/// 按 Java `LocalDateTime.toString()` 的规则格式化：小数秒去掉末尾 0，全为 0 时不输出小数点。
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

/// 统一响应体，与 Java `ApiResponse` 对齐。
pub fn ok<T: Serialize>(data: T) -> Json<Value> {
    Json(serde_json::json!({ "code": 200, "message": "ok", "data": data }))
}

pub fn ok_empty() -> Json<Value> {
    Json(serde_json::json!({ "code": 200, "message": "ok", "data": null }))
}

/// 与 Java `PageResponse` 对齐：content/page/size/totalElements/totalPages。
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
    /// 与 Java `PageResponse.of(List, page, size, totalElements)` 一致：totalPages = ceil(total/size)。
    pub fn of(content: Vec<T>, page: i32, size: i32, total_elements: i64) -> Self {
        let total_pages = if size <= 0 {
            0
        } else {
            ((total_elements as f64) / (size as f64)).ceil() as i32
        };
        Self { content, page, size, total_elements, total_pages }
    }
}

/// 内存分页切片，复刻 Java `PostService#toPage` 的 from/to 计算。
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

/// 与 Java `JsonSupport` 对齐：把 DB 中的 JSON 字符串解析为对象；空 → 空对象。
pub fn json_to_map(raw: Option<&str>) -> Map<String, Value> {
    match raw {
        Some(s) if !s.trim().is_empty() => match serde_json::from_str::<Value>(s) {
            Ok(Value::Object(m)) => m,
            _ => Map::new(),
        },
        _ => Map::new(),
    }
}

/// 解析 JSON 字符串为任意值；空/非法 → Value::Null。
pub fn json_value(raw: Option<&str>) -> Value {
    match raw {
        Some(s) if !s.trim().is_empty() => {
            serde_json::from_str::<Value>(s).unwrap_or(Value::Null)
        }
        _ => Value::Null,
    }
}

/// 解析 JSON 数组为字符串列表；空/非法 → 空列表。
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

/// 解析 JSON 数组为 long 列表；空/非法 → 空列表。
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
