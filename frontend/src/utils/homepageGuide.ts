/** 自定义主页完整示例：装修页与规范文档页共用。 */
export const HOMEPAGE_EXAMPLE_HTML = `<div style="max-width: 760px; margin: 0 auto; padding: 24px 16px;">
  <h1 style="margin: 0 0 8px; font-size: 26px;">你好，欢迎来到我的主页</h1>
  <p style="margin: 0 0 24px; color: #666;">这里展示我的公开资料与近期动态。</p>

  <of-profile heading="关于我"></of-profile>
  <of-stats heading="社区数据"></of-stats>

  <of-posts type="authored" limit="5" heading="我的帖子"></of-posts>
  <of-posts type="liked" limit="5" heading="我点赞的帖子"></of-posts>
  <of-posts type="commented" limit="5" heading="我评论过的帖子"></of-posts>
</div>`

/** 允许使用的 HTML 标签（与后端净化器白名单一致）。 */
export const HOMEPAGE_ALLOWED_TAGS = [
  'a', 'abbr', 'article', 'aside', 'b', 'blockquote', 'br', 'caption', 'code', 'col',
  'colgroup', 'dd', 'del', 'details', 'div', 'dl', 'dt', 'em', 'figcaption', 'figure',
  'footer', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'header', 'hr', 'i', 'img', 'ins', 'li',
  'main', 'mark', 'nav', 'of-posts', 'of-profile', 'of-stats', 'ol', 'p', 'pre', 'q', 's',
  'section', 'small', 'span', 'strong', 'sub', 'summary', 'sup', 'table', 'tbody', 'td',
  'tfoot', 'th', 'thead', 'tr', 'u', 'ul',
]

/** 会被移除的标签及原因。 */
export const HOMEPAGE_BLOCKED_TAGS: { tag: string; reason: string }[] = [
  { tag: 'script', reason: '脚本执行' },
  { tag: 'style', reason: '整段样式表；请改用 style 属性' },
  { tag: 'iframe', reason: '嵌入外部页面' },
  { tag: 'object', reason: '嵌入外部内容' },
  { tag: 'embed', reason: '嵌入外部内容' },
  { tag: 'noscript', reason: '脚本相关' },
  { tag: 'template', reason: '延迟内容' },
  { tag: 'svg', reason: '可携带脚本与外部引用' },
  { tag: 'math', reason: '可携带外部引用' },
  { tag: 'form', reason: '表单提交' },
  { tag: 'input', reason: '表单控件' },
  { tag: 'button', reason: '表单控件' },
  { tag: 'select', reason: '表单控件' },
  { tag: 'textarea', reason: '表单控件' },
  { tag: 'video', reason: '外部媒体' },
  { tag: 'audio', reason: '外部媒体' },
  { tag: 'canvas', reason: '脚本绘制' },
  { tag: 'applet', reason: '已废弃的嵌入内容' },
  { tag: 'frame', reason: '框架页面' },
  { tag: 'frameset', reason: '框架页面' },
  { tag: 'marquee', reason: '已废弃标签' },
  { tag: 'meta', reason: '可触发跳转' },
  { tag: 'link', reason: '外部样式或资源' },
  { tag: 'base', reason: '改变相对地址基准' },
]

/** 允许的内联 CSS 属性（与后端净化器白名单一致）。 */
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
