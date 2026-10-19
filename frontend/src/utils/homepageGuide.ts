/** Complete custom homepage example: shared by the editor and the guidelines page. */
export const HOMEPAGE_EXAMPLE_HTML = `<div style="max-width: 760px; margin: 0 auto; padding: 24px 16px;">
  <h1 style="margin: 0 0 8px; font-size: 26px;">Hello, welcome to my homepage</h1>
  <p style="margin: 0 0 24px; color: #666;">Here you can see my public profile and recent activity.</p>

  <of-profile heading="About me"></of-profile>
  <of-stats heading="Community stats"></of-stats>

  <of-posts type="authored" limit="5" heading="My posts"></of-posts>
  <of-posts type="liked" limit="5" heading="Posts I liked"></of-posts>
  <of-posts type="commented" limit="5" heading="Posts I commented on"></of-posts>
</div>`

/** Allowed HTML tags (matches the backend sanitizer allowlist). */
export const HOMEPAGE_ALLOWED_TAGS = [
  'a', 'abbr', 'article', 'aside', 'b', 'blockquote', 'br', 'caption', 'code', 'col',
  'colgroup', 'dd', 'del', 'details', 'div', 'dl', 'dt', 'em', 'figcaption', 'figure',
  'footer', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'header', 'hr', 'i', 'img', 'ins', 'li',
  'main', 'mark', 'nav', 'of-posts', 'of-profile', 'of-stats', 'ol', 'p', 'pre', 'q', 's',
  'section', 'small', 'span', 'strong', 'sub', 'summary', 'sup', 'table', 'tbody', 'td',
  'tfoot', 'th', 'thead', 'tr', 'u', 'ul',
]

/** Tags that will be removed, with reasons. */
export const HOMEPAGE_BLOCKED_TAGS: { tag: string; reason: string }[] = [
  { tag: 'script', reason: 'Script execution' },
  { tag: 'style', reason: 'Full stylesheet; use the style attribute instead' },
  { tag: 'iframe', reason: 'Embeds external pages' },
  { tag: 'object', reason: 'Embeds external content' },
  { tag: 'embed', reason: 'Embeds external content' },
  { tag: 'noscript', reason: 'Script-related' },
  { tag: 'template', reason: 'Deferred content' },
  { tag: 'svg', reason: 'Can carry scripts and external references' },
  { tag: 'math', reason: 'Can carry external references' },
  { tag: 'form', reason: 'Form submission' },
  { tag: 'input', reason: 'Form control' },
  { tag: 'button', reason: 'Form control' },
  { tag: 'select', reason: 'Form control' },
  { tag: 'textarea', reason: 'Form control' },
  { tag: 'video', reason: 'External media' },
  { tag: 'audio', reason: 'External media' },
  { tag: 'canvas', reason: 'Script-driven drawing' },
  { tag: 'applet', reason: 'Deprecated embedded content' },
  { tag: 'frame', reason: 'Framed page' },
  { tag: 'frameset', reason: 'Framed page' },
  { tag: 'marquee', reason: 'Deprecated tag' },
  { tag: 'meta', reason: 'Can trigger redirects' },
  { tag: 'link', reason: 'External stylesheet or resource' },
  { tag: 'base', reason: 'Changes the base for relative URLs' },
]

/** Allowed inline CSS properties (matches the backend sanitizer allowlist). */
export const HOMEPAGE_CSS_PROPERTIES = [
  'display', 'position', 'top', 'right', 'bottom', 'left', 'z-index', 'float', 'clear',
  'width', 'min-width', 'max-width', 'height', 'min-height', 'max-height',
  'margin', 'margin-top', 'margin-right', 'margin-bottom', 'margin-left',
  'padding', 'padding-top', 'padding-right', 'padding-bottom', 'padding-left',
  'box-sizing', 'overflow', 'overflow-x', 'overflow-y',
  'border', 'border-width', 'border-style', 'border-color',
  'border-top', 'border-right', 'border-bottom', 'border-left', 'border-radius',
  'background', 'background-color', 'background-image', 'background-position',
  'background-size', 'background-repeat', 'background-clip',
  'color', 'opacity', 'box-shadow', 'text-shadow',
  'font', 'font-family', 'font-size', 'font-weight', 'font-style', 'font-variant',
  'line-height', 'letter-spacing', 'word-spacing', 'text-align', 'text-decoration',
  'text-transform', 'text-indent', 'text-overflow', 'white-space', 'word-break',
  'overflow-wrap', 'vertical-align', 'list-style', 'list-style-type', 'list-style-position',
  'flex', 'flex-direction', 'flex-wrap', 'flex-grow', 'flex-shrink', 'flex-basis',
  'justify-content', 'align-items', 'align-content', 'align-self',
  'gap', 'row-gap', 'column-gap', 'order',
  'grid', 'grid-template-columns', 'grid-template-rows', 'grid-auto-flow', 'grid-column',
  'grid-row', 'object-fit', 'object-position', 'aspect-ratio', 'transform',
]
