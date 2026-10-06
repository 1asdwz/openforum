package com.openforum.service;

import org.jsoup.Jsoup;
import org.jsoup.safety.Safelist;
import org.springframework.stereotype.Component;

/**
 * Sanitizes owner-submitted custom page HTML. Disallowed tags (script/style/iframe/object/embed/form
 * and friends), event handler attributes and unsafe protocols (e.g. {@code javascript:}) are removed.
 */
@Component
public class HtmlSanitizer {

    private final Safelist safelist;

    public HtmlSanitizer() {
        this.safelist = Safelist.relaxed()
                .removeTags("script", "style", "iframe", "object", "embed", "form", "input", "button",
                        "link", "meta", "base", "noscript")
                .removeProtocols("a", "href", "ftp")
                .removeProtocols("img", "src", "ftp");
    }

    public String sanitize(String html) {
        if (html == null || html.isBlank()) {
            return "";
        }
        return Jsoup.clean(html, safelist);
    }
}
