use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// Base allowlisted tags (shared by both homepage types).
fn base_tags() -> HashSet<&'static str> {
    HashSet::from([
        "a", "abbr", "article", "aside", "b", "blockquote", "br", "caption", "code", "col",
        "colgroup", "dd", "del", "details", "div", "dl", "dt", "em", "figcaption", "figure",
        "footer", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hr", "i", "img", "ins", "li",
        "main", "mark", "nav", "ol", "p", "pre", "q", "s", "section", "small", "span", "strong",
        "sub", "summary", "sup", "table", "tbody", "td", "tfoot", "th", "thead", "tr", "u", "ul",
    ])
}

/// Tags removed together with their content.
fn clean_content_tags() -> HashSet<&'static str> {
    HashSet::from([
        "script", "style", "iframe", "object", "embed", "noscript", "template", "svg", "math",
        "form", "input", "button", "select", "textarea", "video", "audio", "canvas", "applet",
        "frame", "frameset", "marquee", "meta", "link", "base",
    ])
}

/// Generic attribute allowlist.
fn generic_attributes() -> HashSet<&'static str> {
    HashSet::from(["class", "id", "title", "dir", "lang", "style"])
}

/// Strict sanitization for user homepage HTML; keeps user homepage data components.
pub fn sanitize_user_homepage(html: &str) -> String {
    sanitize_component_page(
        html,
        &[
            ("of-profile", &["heading"]),
            ("of-stats", &["heading"]),
            ("of-posts", &["type", "limit", "heading"]),
        ],
    )
}

/// Strict sanitization for custom forum homepage HTML; keeps forum data components.
pub fn sanitize_forum_homepage(html: &str) -> String {
    sanitize_component_page(
        html,
        &[
            ("of-forum", &["heading"]),
            ("of-forum-stats", &["heading"]),
            ("of-forum-posts", &["type", "limit", "heading"]),
            ("of-members", &["limit", "heading"]),
        ],
    )
}

/// Compatibility for legacy callers: equivalent to `sanitize_user_homepage`.
pub fn sanitize_homepage(html: &str) -> String {
    sanitize_user_homepage(html)
}

/// Shared sanitization pipeline: base allowlist plus specified data components and their attributes.
fn sanitize_component_page(html: &str, components: &[(&'static str, &[&'static str])]) -> String {
    if html.trim().is_empty() {
        return String::new();
    }

    let mut tags = base_tags();
    let mut tag_attributes: HashMap<&'static str, HashSet<&'static str>> = HashMap::new();
    tag_attributes.insert("a", HashSet::from(["href", "title"]));
    tag_attributes.insert("img", HashSet::from(["src", "alt", "width", "height"]));
    tag_attributes.insert("td", HashSet::from(["colspan", "rowspan"]));
    tag_attributes.insert("th", HashSet::from(["colspan", "rowspan"]));
    tag_attributes.insert("col", HashSet::from(["span"]));
    tag_attributes.insert("colgroup", HashSet::from(["span"]));
    tag_attributes.insert("ol", HashSet::from(["start"]));
    tag_attributes.insert("li", HashSet::from(["value"]));
    tag_attributes.insert("details", HashSet::from(["open"]));
    for (tag, attrs) in components {
        tags.insert(tag);
        tag_attributes.insert(tag, attrs.iter().copied().collect());
    }

    let mut url_schemes: HashSet<&'static str> = HashSet::new();
    url_schemes.extend(["http", "https", "mailto"]);

    let mut builder = ammonia::Builder::default();
    builder
        .tags(tags)
        .clean_content_tags(clean_content_tags())
        .generic_attributes(generic_attributes())
        .tag_attributes(tag_attributes)
        .url_schemes(url_schemes)
        .link_rel(Some("noopener noreferrer nofollow"))
        .filter_style_properties(css_properties())
        .attribute_filter(|element, attribute, value| {
            if attribute.eq_ignore_ascii_case("style") {
                return sanitize_css_declarations(value).map(Cow::Owned);
            }
            if element == "img" && attribute.eq_ignore_ascii_case("src") {
                return is_site_media_src(value).then_some(Cow::Borrowed(value));
            }
            if element == "a" && attribute.eq_ignore_ascii_case("href") {
                return is_safe_href(value).then_some(Cow::Borrowed(value));
            }
            if element.starts_with("of-") {
                if attribute.eq_ignore_ascii_case("type") {
                    if element == "of-forum-posts" {
                        return normalize_forum_post_type(value).map(Cow::Owned);
                    }
                    if element == "of-posts" {
                        return normalize_user_post_type(value).map(Cow::Owned);
                    }
                    return None;
                }
                if attribute.eq_ignore_ascii_case("limit") {
                    return normalize_component_limit(value).map(Cow::Owned);
                }
                if attribute.eq_ignore_ascii_case("heading") {
                    return normalize_component_heading(value).map(Cow::Owned);
                }
            }
            if value.chars().any(char::is_control) {
                return None;
            }
            Some(Cow::Borrowed(value))
        });

    builder.clean(html).to_string()
}

/// Image sources must be site media proxy paths: `/forum/api/media/{numeric id}`.
fn is_site_media_src(value: &str) -> bool {
    let Some(rest) = value.trim().strip_prefix("/forum/api/media/") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

/// Links may only be http/https/mailto, single-slash-prefixed site paths, or in-page anchors.
fn is_safe_href(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    if value.starts_with('#') {
        return true;
    }
    if value.starts_with('/') {
        return !value.starts_with("//");
    }
    let lower = value.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:")
}

/// List type for the user homepage posts component; the attribute is dropped for invalid values.
fn normalize_user_post_type(value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    matches!(value.as_str(), "authored" | "liked" | "commented").then_some(value)
}

/// List type for the forum homepage posts component; the attribute is dropped for invalid values.
fn normalize_forum_post_type(value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    matches!(value.as_str(), "latest" | "hot").then_some(value)
}

/// Item count for data components: accepts only decimal integers from 1 to 10.
fn normalize_component_limit(value: &str) -> Option<String> {
    let value = value.trim();
    let limit: u32 = value.parse().ok()?;
    (1..=10).contains(&limit).then(|| limit.to_string())
}

/// Heading for data components: 1–30 characters with no control characters.
fn normalize_component_heading(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 30 || value.chars().any(char::is_control) {
        return None;
    }
    Some(value.to_string())
}

/// Inline CSS property allowlist.
fn css_properties() -> HashSet<&'static str> {
    HashSet::from([
        "display",
        "position",
        "top",
        "right",
        "bottom",
        "left",
        "z-index",
        "float",
        "clear",
        "width",
        "min-width",
        "max-width",
        "height",
        "min-height",
        "max-height",
        "margin",
        "margin-top",
        "margin-right",
        "margin-bottom",
        "margin-left",
        "padding",
        "padding-top",
        "padding-right",
        "padding-bottom",
        "padding-left",
        "box-sizing",
        "overflow",
        "overflow-x",
        "overflow-y",
        "border",
        "border-width",
        "border-style",
        "border-color",
        "border-top",
        "border-right",
        "border-bottom",
        "border-left",
        "border-radius",
        "background",
        "background-color",
        "background-image",
        "background-position",
        "background-size",
        "background-repeat",
        "background-clip",
        "color",
        "opacity",
        "box-shadow",
        "text-shadow",
        "font",
        "font-family",
        "font-size",
        "font-weight",
        "font-style",
        "font-variant",
        "line-height",
        "letter-spacing",
        "word-spacing",
        "text-align",
        "text-decoration",
        "text-transform",
        "text-indent",
        "text-overflow",
        "white-space",
        "word-break",
        "overflow-wrap",
        "vertical-align",
        "list-style",
        "list-style-type",
        "list-style-position",
        "flex",
        "flex-direction",
        "flex-wrap",
        "flex-grow",
        "flex-shrink",
        "flex-basis",
        "justify-content",
        "align-items",
        "align-content",
        "align-self",
        "gap",
        "row-gap",
        "column-gap",
        "order",
        "grid",
        "grid-template-columns",
        "grid-template-rows",
        "grid-auto-flow",
        "grid-column",
        "grid-row",
        "object-fit",
        "object-position",
        "aspect-ratio",
        "transform",
    ])
}

const MAX_CSS_BYTES: usize = 8 * 1024;
const MAX_DECLARATION_CHARS: usize = 512;

/// Filter the style attribute, dropping invalid declarations one by one; remove the entire attribute when no valid declarations remain.
pub fn sanitize_css_declarations(input: &str) -> Option<String> {
    if input.len() > MAX_CSS_BYTES {
        return None;
    }
    // CSS escapes can bypass string checks such as `url(`, so reject backslashes outright.
    if input.contains('\\') {
        return None;
    }
    let stripped = strip_css_comments(input)?;
    let allowed = css_properties();
    let mut out: Vec<String> = Vec::new();

    for declaration in split_declarations(&stripped) {
        let Some((raw_prop, raw_value)) = declaration.split_once(':') else {
            continue;
        };
        let prop = raw_prop.trim().to_ascii_lowercase();
        if prop.is_empty()
            || !prop.chars().all(|c| c.is_ascii_lowercase() || c == '-')
            || !allowed.contains(prop.as_str())
        {
            continue;
        }
        let value = raw_value.trim();
        if value.is_empty()
            || value.chars().count() > MAX_DECLARATION_CHARS
            || has_unsafe_css_value(value)
        {
            continue;
        }
        out.push(format!("{prop}:{value}"));
    }

    if out.is_empty() {
        None
    } else {
        Some(out.join(";"))
    }
}

fn has_unsafe_css_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    const FORBIDDEN: [&str; 13] = [
        "url(",
        "image-set(",
        "cross-fade(",
        "element(",
        "paint(",
        "expression(",
        "javascript:",
        "vbscript:",
        "@import",
        "behavior",
        "-moz-binding",
        "var(",
        "/*",
    ];
    if value.contains('\\') || value.contains('<') || value.contains('>') {
        return true;
    }
    FORBIDDEN.iter().any(|needle| lower.contains(needle))
}

/// Strip CSS comments; returns None if a comment is unterminated (the whole style is dropped).
fn strip_css_comments(input: &str) -> Option<String> {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            let end = input[i + 2..].find("*/")?;
            i = i + 2 + end + 2;
            continue;
        }
        let ch = input[i..].chars().next()?;
        out.push(ch);
        i += ch.len_utf8();
    }
    Some(out)
}

/// Split declarations on semicolons, ignoring semicolons inside parentheses and quotes.
fn split_declarations(input: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    for (i, ch) in input.char_indices() {
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            ';' if depth == 0 => {
                out.push(&input[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&input[start..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_script_and_its_content() {
        let out = sanitize_homepage("<p>hi</p><script>alert(1)</script>");
        assert!(out.contains("<p>hi</p>"));
        assert!(!out.contains("script"));
        assert!(!out.contains("alert"));
    }

    #[test]
    fn removes_event_handlers_and_invalid_img_src() {
        let out = sanitize_homepage("<img src=x onerror=alert(1)>");
        assert!(!out.contains("onerror"));
        assert!(!out.contains("src="));
    }

    #[test]
    fn keeps_only_site_media_images() {
        let kept = sanitize_homepage(r#"<img src="/forum/api/media/12" alt="a">"#);
        assert!(kept.contains("/forum/api/media/12"));

        for bad in [
            r#"<img src="https://evil.example/a.png">"#,
            r#"<img src="//evil.example/a.png">"#,
            r#"<img src="data:image/png;base64,AAAA">"#,
            r#"<img src="/forum/api/media/12/../../evil">"#,
        ] {
            let out = sanitize_homepage(bad);
            assert!(!out.contains("evil.example"), "{bad} -> {out}");
            assert!(!out.contains("data:"), "{bad} -> {out}");
            assert!(!out.contains("src="), "{bad} -> {out}");
        }
    }

    #[test]
    fn link_protocols_are_restricted() {
        let js = sanitize_homepage(r#"<a href="javascript:alert(1)">x</a>"#);
        assert!(!js.contains("javascript"));

        let rel = sanitize_homepage(r#"<a href="//evil.example">x</a>"#);
        assert!(!rel.contains("evil.example"));

        let data = sanitize_homepage(r#"<a href="data:text/html,<script>">x</a>"#);
        assert!(!data.contains("data:"));

        let external = sanitize_homepage(r#"<a href="https://example.com/a">x</a>"#);
        assert!(external.contains(r#"href="https://example.com/a""#));
        assert!(external.contains("noopener noreferrer nofollow"));
    }

    #[test]
    fn removes_forms_iframes_svg_and_base() {
        for bad in [
            r#"<form action="/x"><input name="a"></form>"#,
            r#"<iframe src="https://evil.example"></iframe>"#,
            r#"<svg onload="alert(1)"></svg>"#,
            r#"<base href="https://evil.example/">"#,
            r#"<meta http-equiv="refresh" content="0;url=https://evil.example">"#,
            r#"<link rel="stylesheet" href="https://evil.example/a.css">"#,
            r#"<style>body{display:none}</style>"#,
        ] {
            let out = sanitize_homepage(bad);
            assert!(!out.contains("evil.example"), "{bad} -> {out}");
            assert!(!out.contains("<form"), "{bad} -> {out}");
            assert!(!out.contains("<iframe"), "{bad} -> {out}");
            assert!(!out.contains("<svg"), "{bad} -> {out}");
            assert!(!out.contains("<base"), "{bad} -> {out}");
            assert!(!out.contains("<meta"), "{bad} -> {out}");
            assert!(!out.contains("<link"), "{bad} -> {out}");
            assert!(!out.contains("<style"), "{bad} -> {out}");
            assert!(!out.contains("display:none"), "{bad} -> {out}");
        }
    }

    #[test]
    fn css_url_and_import_are_removed() {
        let url = sanitize_homepage(
            r#"<div style="background:url(https://evil.example/a.png);color:red">x</div>"#,
        );
        assert!(!url.contains("url("));
        assert!(!url.contains("evil.example"));
        assert!(url.contains("color:red"));

        let import = sanitize_homepage(r#"<div style="@import url(https://evil.example/a.css)">x</div>"#);
        assert!(!import.contains("@import"));
        assert!(!import.contains("evil.example"));

        let escaped = sanitize_homepage(r#"<div style="background:u\72l(https://evil.example/a.png)">x</div>"#);
        assert!(!escaped.contains("style="), "{escaped}");
    }

    #[test]
    fn css_allowlist_keeps_safe_declarations() {
        let out = sanitize_homepage(
            r#"<div style="color:red;position:fixed;animation:spin 1s;unknown:1">x</div>"#,
        );
        assert!(out.contains("color:red"));
        assert!(out.contains("position:fixed"));
        assert!(!out.contains("animation"));
        assert!(!out.contains("unknown"));
    }

    #[test]
    fn keeps_component_tags_and_valid_attributes() {
        let out = sanitize_homepage(
            r#"<of-profile heading="About me"></of-profile><of-stats heading="Stats"></of-stats><of-posts type="liked" limit="10" heading="Posts I liked">Loading</of-posts>"#,
        );
        assert!(out.contains("<of-profile"), "{out}");
        assert!(out.contains("<of-stats"), "{out}");
        assert!(out.contains(r#"type="liked""#), "{out}");
        assert!(out.contains(r#"limit="10""#), "{out}");
        assert!(out.contains("Posts I liked"), "{out}");
        assert!(out.contains("Loading"), "{out}");
    }

    #[test]
    fn drops_invalid_component_attributes() {
        let out = sanitize_homepage(
            r#"<of-posts type="evil" limit="999" heading="This heading is intentionally longer than thirty characters for testing" onclick="alert(1)">x</of-posts>"#,
        );
        assert!(!out.contains("type="), "{out}");
        assert!(!out.contains("limit="), "{out}");
        assert!(!out.contains("heading="), "{out}");
        assert!(!out.contains("onclick"), "{out}");

        for bad in [r#"<of-posts limit="0">x</of-posts>"#, r#"<of-posts limit="11">x</of-posts>"#, r#"<of-posts limit="abc">x</of-posts>"#] {
            let out = sanitize_homepage(bad);
            assert!(!out.contains("limit="), "{bad} -> {out}");
        }
    }

    #[test]
    fn unknown_of_tags_are_stripped() {
        let out = sanitize_homepage("<of-foo>hello</of-foo>");
        assert!(!out.contains("of-foo"), "{out}");
        assert!(out.contains("hello"), "{out}");
    }

    #[test]
    fn malformed_input_does_not_panic() {
        let out = sanitize_homepage("<div><p><a href=\"/x\">hello");
        assert!(!out.contains("script"));
        assert!(out.contains("hello"));
    }

    #[test]
    fn forum_homepage_keeps_forum_components() {
        let out = sanitize_forum_homepage(
            r#"<of-forum heading="Forum info"></of-forum><of-forum-stats heading="Stats"></of-forum-stats><of-forum-posts type="hot" limit="8" heading="Popular posts"></of-forum-posts><of-members limit="6" heading="Members"></of-members>"#,
        );
        assert!(out.contains("<of-forum "), "{out}");
        assert!(out.contains("<of-forum-stats "), "{out}");
        assert!(out.contains(r#"type="hot""#), "{out}");
        assert!(out.contains(r#"limit="8""#), "{out}");
        assert!(out.contains("<of-members "), "{out}");
        assert!(out.contains(r#"limit="6""#), "{out}");
    }

    #[test]
    fn forum_homepage_removes_user_components() {
        let out = sanitize_forum_homepage(
            "<of-profile>Profile</of-profile><of-stats>Stats</of-stats><of-posts>Posts</of-posts>",
        );
        assert!(!out.contains("of-profile"), "{out}");
        assert!(!out.contains("<of-stats"), "{out}");
        assert!(!out.contains("<of-posts"), "{out}");
        assert!(out.contains("Profile"), "{out}");
        assert!(out.contains("Stats"), "{out}");
        assert!(out.contains("Posts"), "{out}");
    }

    #[test]
    fn user_homepage_removes_forum_components() {
        let out = sanitize_user_homepage(
            "<of-forum>Forum</of-forum><of-forum-stats>Stats</of-forum-stats><of-forum-posts>Posts</of-forum-posts><of-members>Members</of-members>",
        );
        for tag in ["<of-forum", "<of-forum-stats", "<of-forum-posts", "<of-members"] {
            assert!(!out.contains(tag), "{tag} -> {out}");
        }
        assert!(out.contains("Forum"), "{out}");
        assert!(out.contains("Members"), "{out}");
    }

    #[test]
    fn forum_homepage_drops_invalid_component_attributes() {
        let out = sanitize_forum_homepage(
            r#"<of-forum-posts type="evil" limit="12" heading="This heading is intentionally longer than thirty characters for testing" onclick="alert(1)">x</of-forum-posts><of-members limit="0">y</of-members>"#,
        );
        assert!(!out.contains("type="), "{out}");
        assert!(!out.contains("limit="), "{out}");
        assert!(!out.contains("heading="), "{out}");
        assert!(!out.contains("onclick"), "{out}");

        let latest = sanitize_forum_homepage(r#"<of-forum-posts type="latest" limit="1">x</of-forum-posts>"#);
        assert!(latest.contains(r#"type="latest""#), "{latest}");
        assert!(latest.contains(r#"limit="1""#), "{latest}");
    }
}
