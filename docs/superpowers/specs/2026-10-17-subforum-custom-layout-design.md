# 子论坛「默认模板 / 自定义代码」设计稿

- 日期：2026-10-17
- 状态：草案，待评审
- 关联：`2026-10-03-openforum-design.md` §7、`2026-10-08-forum-layout-design.md`
- 范围：子论坛创建者对子论坛首页（板块布局与样式）的两种设计模式及安全边界

---

## 1. 需求与现状

### 1.1 需求
子论坛创建者（坛主）可以二选一：
1. **选择默认论坛模板**：零代码，填配置即可（公告 / 板块 / 链接）。
2. **自行编写代码**：用 HTML + CSS 自行设计子论坛的板块布局与视觉样式。
3. 两种方式都必须**在安全的前提下**（防 XSS、防沙箱逃逸、防钓鱼、防资源耗尽）。

### 1.2 现状盘点（已有实现）
| 能力 | 位置 | 说明 |
|---|---|---|
| `template_type` = `DEFAULT`\|`CUSTOM` | `SubForum` 实体 / `TemplateType` | 已有枚举 |
| 默认模板配置 `template_config` (JSON) | `ForumSettings.vue` `template` tab | announcement / sections / links |
| 自定义 HTML `custom_page_html` | `SubForum` 实体 L54 | LONGTEXT |
| 审核状态机 `custom_page_status` | `CustomPageStatus` | DRAFT/PENDING/APPROVED/REJECTED |
| 后端净化 | `HtmlSanitizer.java`（jsoup）/ `sanitizer.rs`（ammonia） | 白名单净化 |
| 审核接口 | `PUT /forums/{id}/custom-page`、`/submit`、后台 `/admin/custom-pages` | 已有 |
| 渲染 | `CustomPageRenderer.vue` | `iframe sandbox="allow-popups"` + `srcdoc` |
| 仅 APPROVED 对外 | `build_detail` / `SubForumService` | 未过审返回 null |

### 1.3 差距（本方案要补的）
- ❌ 净化器**删除 `<style>` 与内联样式**，坛主无法「设计样式」——与需求 2 直接冲突。
- ❌ 只有**整页一大块 HTML**，没有「板块级」代码单元。
- ❌ 没有**版本历史 / 回滚 / diff**，审核无法对照线上版本。
- ❌ iframe 内**没有内联 CSP**，主站也未见 CSP 响应头。
- ❌ 没有**大小上限、提交限流、审计日志**。
- ❌ `srcdoc` 未注入 `<base target="_blank">`，外链存在同窗钓鱼风险。

---

## 2. 总体方案：双模式

```
坛主设置页
   │
   ├─ 模式 A：默认模板 (template_type = DEFAULT)
   │     结构化配置 → 前端用站内组件渲染（无第三方代码，最安全）
   │
   └─ 模式 B：自定义代码 (template_type = CUSTOM)
         分栏编辑器：HTML + CSS（禁止 JS）
           ├─ 保存草稿 (DRAFT)
           ├─ 提交审核 (PENDING)
           └─ 审核通过 (APPROVED) → 沙箱 iframe 对外渲染
                └─ 未通过 / 未提交 → 自动回退模式 A
```

- 两种模式**互斥但可随时切换**，回退逻辑已存在（`build_detail` 只在 APPROVED 时返回 html）。
- CUSTOM 模式下，`custom_page_html` 与新增的 `custom_page_css` 一起在沙箱 iframe 内组装成完整文档。
- **不允许任何 JavaScript**：坛主「写代码」的产出物是 HTML(结构) + CSS(样式)，静态化。需要动态数据时，用**服务端占位符**（§7.3）而非脚本。

---

## 3. 安全模型（核心）

采用**纵深防御**：提交 → 服务端清洗（权威）→ 审核 → 沙箱渲染 → 运行时 CSP → 审计。

### 3.1 第一层：提交入口
- 未登录 / 非坛主 / 非管理员拒绝（复用 `access::can_manage`）。
- **限流**：同一论坛「保存草稿」≤ 20 次/分钟，「提交审核」≤ 5 次/小时（防止刷审、刷日志）。
- **大小上限**：HTML ≤ 64 KB，CSS ≤ 32 KB；超限 400。

### 3.2 第二层：服务端清洗（不可绕过，以服务端为准）
> 前端预览可宽松，但**入库/对外的一定是服务端清洗后的结果**。

**标签白名单**（HTML）：
`h1-h6, p, a, img, ul, ol, li, blockquote, pre, code, hr, br, strong, em, u, s, span, div, section, header, footer, nav, main, aside, article, figure, figcaption, table, thead, tbody, tr, th, td, video, source, picture`

**显式移除**：`script, iframe, object, embed, form, input, button, select, textarea, link, meta, base, noscript, style`（`<style>` 不进 HTML 字段，样式走独立 CSS 字段）。

**属性白名单**：`class, id, href, src, alt, title, width, height, target, rel, colspan, rowspan, controls, poster, loading`。
- **剥离全部 `on*` 事件属性**（jsoup/ammonia 默认行为，保留）。
- `target` 强制覆盖为 `_blank`，`rel` 强制追加 `noopener noreferrer nofollow`。

**协议白名单**：
- `a.href`：`http, https, mailto`
- `img.src` / `video.poster`：`http, https, data:image/(png|jpeg|gif|webp)`
- 其余（`javascript:`、`data:text/html`、`vbscript:`、`ftp`）一律拒绝。

**CSS 清洗**（新增，关键）：
- 用 CSS 解析器（Java：`ph-css` / Rust：`lightningcss`）解析为 AST，逐条声明校验后**重新序列化**（禁止原样透传）。
- **属性白名单**：`color, background-color, background-image, background, border*, padding*, margin*, font*, text-align, line-height, letter-spacing, width, height, max/min-*, display(flex/inline-flex/block/inline-block/grid/none), flex*, grid*, gap, align-*, justify-*, box-shadow, border-radius, opacity, object-fit, transition`（transition 允许，但不可配合脚本产生可用交互）。
- **禁止**：`position:fixed/sticky` 之外的仍允许，但**禁止 `url()` 的 `javascript:`/`data:text/html`**、禁止 `expression()`、`@import`、`behavior`、`-moz-binding`、`behavior`。
- **选择器限制**：禁止 `html`/`body` 级全局重写（避免遮盖沙箱外的观感错觉），不改 `pointer-events:none`；允许 `:hover` 等纯视觉伪类。
- **禁止外链资源夹带**：`background-image` / `@font-face` 的 URL 仅允许白名单 CDN 或本站 MinIO（见 §3.5）。

### 3.3 第三层：审核（人 + AI）
- 复用现有 `DRAFT → PENDING → APPROVED/REJECTED`。
- AI 预审（可插拔 `ReviewPlugin`）：检测疑似钓鱼文案（"请输入密码/验证码"）、冒充官方 UI、外链跳转诱导。
- 人工审核台 `/admin/custom-pages`：展示**净化后 HTML + CSS + 渲染预览 + 与线上版本 diff**。

### 3.4 第四层：渲染隔离（前端）
```html
<iframe
  sandbox="allow-popups allow-popups-to-escape-sandbox"
  referrerpolicy="no-referrer"
  :srcdoc="assembledDoc"
/>
```
- **不包含 `allow-scripts`**：iframe 内 JavaScript 完全禁用，即使清洗漏网也执行不了。
- **不包含 `allow-same-origin`**：与主站 DOM / Cookie / localStorage 隔离。
- `assembledDoc` 由平台拼装，坛主无法控制外层：
  ```html
  <!doctype html>
  <html><head>
    <meta charset="utf-8">
    <base target="_blank">
    <meta http-equiv="Content-Security-Policy"
          content="default-src 'none'; img-src https: data:; media-src https:;
                   style-src 'unsafe-inline'; font-src https:; form-action 'none';
                   base-uri 'none'; frame-ancestors 'self'">
    <style>/* 清洗后的 CSS */</style>
  </head><body><!-- 清洗后的 HTML --></body></html>
  ```
- **不通过 postMessage 传递敏感数据**（无 script 也不需要）；确需数据用服务端占位符（§7.3）。

### 3.5 第五层：运行时与资源
- 主站可对 `/forum/` 增加 `Content-Security-Policy` 响应头，`frame-src 'self'`（限定自定义页只能内嵌同源）。
- 外链图片/字体：走本站 MinIO 代理或白名单域名，避免引用第三方追踪像素。
- iframe 默认高度自适应 + 上限（如 4000px），防止超长页面。

### 3.6 威胁模型
| 威胁 | 载体 | 防御 |
|---|---|---|
| 存储型 XSS | `<script>`、`on*`、`javascript:` | 服务端白名单净化 + iframe 无 `allow-scripts` |
| 沙箱逃逸 | 依赖浏览器漏洞 | 双重隔离（无 script + 无 same-origin）+ CSP |
| 点击劫持 | 全屏覆盖层 | iframe 无 `allow-scripts`；主站 CSP `frame-ancestors` |
| 钓鱼跳转 | `<a target="_top">` | `base target=_blank` + 强制 `rel=noopener` |
| CSS 注入 | `expression()` / `url(javascript:)` | CSS 解析白名单重序列化 |
| 数据外泄 | 伪装图片 URL 回传 | 外链域名白名单 / 同源代理 |
| 资源耗尽 | 超大 HTML/CSS、海量 DOM | 大小上限 + 限流 + 渲染高度上限 |
| 越权改页 | 非坛主调用接口 | `can_manage` 校验（已有） |
| 审核绕过 | 直接写库 | 对外只认 `APPROVED`（已有） |

---

## 4. 数据模型演进

### 4.1 `sub_forums`（扩展）
```
+ custom_page_css      LONGTEXT        -- 自定义样式（清洗后）
+ custom_page_version  INT DEFAULT 0   -- 当前线上版本号
+ custom_page_updated_at DATETIME
```

### 4.2 新增 `forum_page_versions`（版本历史 / 回滚 / diff）
```
id                BIGINT PK
forum_id          BIGINT FK -> sub_forums
version           INT              -- 自增，按 forum 维度
html              LONGTEXT         -- 清洗后
css               LONGTEXT         -- 清洗后
raw_html          LONGTEXT         -- 坛主原始输入（仅审核/取证可见）
raw_css           LONGTEXT
status            VARCHAR(16)      -- DRAFT/PENDING/APPROVED/REJECTED
review_note       VARCHAR(512)
created_by        BIGINT FK -> users
created_at        DATETIME
UNIQUE(forum_id, version)
```
- 每次「保存草稿」生成新版本；审核针对**具体版本**；APPROVED 时把该版本内容同步回 `sub_forums`。

### 4.3 `template_config` 向后兼容
保留现有 `{announcement, sections[], links[]}`；后续可扩展 `blocks[]`（见 §6.1），前端做兼容解析。

---

## 5. API 契约（在现有基础上扩展）

| 方法 | 路径 | 权限 | 说明 |
|---|---|---|---|
| GET | `/forums/{slug}` | 公开 | 详情；`customPageHtml`/`customPageCss` 仅 APPROVED 返回（已有） |
| PUT | `/forums/{id}/custom-page` | 坛主 | body `{html, css}` → 清洗 → 生成 DRAFT 版本（**改造**：增加 css） |
| POST | `/forums/{id}/custom-page/submit` | 坛主 | 当前草稿 → PENDING |
| GET | `/forums/{id}/custom-page` | 坛主/管理员 | 返回 `{html, css, status, reviewNote, version}` |
| GET | `/forums/{id}/custom-page/versions` | 坛主/管理员 | 分页版本列表 |
| GET | `/forums/{id}/custom-page/versions/{v}` | 坛主/管理员 | 单版本详情（含 raw 仅管理员） |
| POST | `/forums/{id}/custom-page/restore` | 坛主 | body `{version}` → 拉旧版为新的 DRAFT |
| GET | `/admin/custom-pages` | 管理员 | 待审列表（已有） |
| POST | `/admin/custom-pages/{id}/approve` \| `/reject` | 管理员 | 审核（已有，改为对版本操作） |

统一响应沿用 `{code, message, data}`；错误沿用 `ApiException` 语义（400/401/403/404）。

---

## 6. 前端设计

### 6.1 模式 A：默认模板（可视化，无代码）
- 现有 `template` tab 升级为**块编辑器**：公告 / 板块(标题+内容) / 链接 / 图片 / 富文本块，可拖拽排序。
- 底部提供 **3 套预置主题配色**（跟随 `themeColor`），产出仍存 `template_config`。
- 渲染走站内 `ForumTemplate.vue` 组件，**不涉及第三方 HTML**。

### 6.2 模式 B：自定义代码编辑器
- 选择 `CUSTOM` 后出现「自定义代码」tab：
  - **分栏**：左代码（HTML / CSS 两个子页签），右**实时预览** iframe（复用同一沙箱逻辑）。
  - 代码编辑器：`CodeMirror 6`（轻量，HTML/CSS 高亮），替换现有裸 `el-input textarea`。
  - 顶部：状态标签（草稿/审核中/已通过/已驳回）+ 审核意见 + 版本号。
  - 操作：`保存草稿` / `提交审核` / `版本历史` / `回滚`。
- **模板库**：提供 3~5 个安全起始模板（极简 / 卡片 / 博客 / 活动页），一键套用后可改。
- **校验提示**：前端本地用 DOMPurify + 规则做「预检」，标出将被移除的标签/属性，减少提交后被驳回。

### 6.3 数据占位符（安全的动态内容）
- 坛主可在 HTML 里写 `{{forum.name}}`、`{{forum.memberCount}}`、`{{forum.postCount}}`、`{{forum.description}}`。
- 服务端在清洗**之后**做字符串替换，值经 HTML 转义再插入（防注入）。
- 仅支持固定占位符集合，**不开放表达式/函数**，不做模板引擎执行。

---

## 7. 渲染管线

```
坛主输入 (raw_html, raw_css)
   ↓ 服务端 sanitizeHtml() + sanitizeCss()
   ↓ 落库 forum_page_versions (DRAFT)
   ↓ 提交 → PENDING → 审核 → APPROVED
   ↓ 同步回 sub_forums (custom_page_html/css, version, status)
GET /forums/{slug}
   ↓ 仅 APPROVED 下发 html/css
前端 assembleDoc(html, css, forumData)
   ↓ 服务端占位符替换（或前端安全替换 + 转义）
   ↓ 拼装 <base> + CSP meta + <style> + body
   ↓ <iframe sandbox="allow-popups"> srcdoc
```

---

## 8. 实施计划

**Phase 0（安全底座，优先）**
1. 拆分 `custom_page_css` 字段 + CSS 清洗器（ph-css / lightningcss）。
2. iframe 组装函数统一到 `assembleDoc()`：注入 `<base>`、CSP meta、`<style>`。
3. `target=_blank` + `rel=noopener` 强制。
4. 大小上限 + 提交限流。
- 验收：现有审核流不变；CSS 生效且恶意 CSS 被拦。

**Phase 1（创作体验）**
5. `forum_page_versions` 表 + 版本/回滚/diff 接口。
6. CodeMirror 分栏编辑器 + 实时预览 + 模板库。
7. 审核台展示净化后 HTML/CSS + 预览 + diff。

**Phase 2（增强）**
8. 默认模板升级为块编辑器（拖拽）。
9. AI 预审（钓鱼/冒充）接入 `ReviewPlugin`。
10. 占位符扩展到 `{{latestPosts}}` 等（服务端安全渲染为 HTML 片段）。

---

## 9. 验收 / 测试用例

| 用例 | 期望 |
|---|---|
| HTML 含 `<script>alert(1)</script>` | 入库被移除，页面无脚本 |
| 属性 `onclick="..."` | 被剥离 |
| `<a href="javascript:...">` | 协议被拒 |
| `<a target="_top">` | 被改为 `_blank` 且加 `rel=noopener` |
| CSS `background:url(javascript:...)` | 被拒 |
| CSS `position:fixed` 全屏遮罩 | 按策略处理（禁止或限制） |
| HTML 70KB | 400 超限 |
| 1 分钟内保存 30 次 | 后 10 次 429/限流 |
| 非坛主调用保存接口 | 403 |
| 已提交未过审访问子论坛 | 回退默认模板 |
| 回滚到 v3 | sub_forums 同步为 v3 内容，重新进入审核 |

---

## 10. 非目标（本方案不做）
- 允许坛主执行任意 JavaScript（明确禁止）。
- 服务端模板引擎（Thymeleaf/Freemarker 用户可编辑）——安全风险过高，用固定占位符替代。
- 坛主上传任意字体/脚本资源。
- 独立域名托管自定义页。

---

## 11. 与现有实现的改动清单
| 文件 | 改动 |
|---|---|
| `backend/.../service/HtmlSanitizer.java` | 增加 `sanitizeCss()`；HTML 白名单微调 |
| `backend/.../entity/SubForum.java` | 加 `custom_page_css` 等字段 |
| `backend/.../entity/ForumPageVersion.java`（新） | 版本实体 |
| `backend/.../controller/SubForumController.java` | 版本/回滚接口 |
| `backend-rs/src/sanitizer.rs` | 对齐 CSS 清洗 |
| `backend-rs/src/routes/forums.rs` | 保存接口加 css + 版本 |
| `frontend/.../components/CustomPageRenderer.vue` | 抽取 `assembleDoc()` + CSP + base |
| `frontend/.../views/ForumSettings.vue` | CodeMirror 分栏编辑器 + 版本/模板库 |
| `frontend/.../views/Admin.vue` | 审核台加 CSS 与 diff |
