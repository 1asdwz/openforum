/// 清洗坛主提交的自定义页面 HTML，语义对齐 Java `HtmlSanitizer`（jsoup `Safelist.relaxed()`）。
///
/// ammonia 默认策略已移除 script/style/iframe/object/embed/form 等危险标签，
/// 同时剥离 on* 事件属性与 `javascript:` 等不安全协议。
pub fn sanitize(html: &str) -> String {
    if html.trim().is_empty() {
        return String::new();
    }
    ammonia::clean(html)
}
