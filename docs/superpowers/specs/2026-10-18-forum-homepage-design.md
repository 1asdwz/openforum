# 自定义论坛首页设计规格

- 日期：2026-10-18
- 状态：待评审
- 范围：Rust 后端（`backend-rs`）+ Vue 前端（`frontend`）
- 关联规格：
  - [自定义个人主页设计规格](2026-10-18-user-homepage-design.md)
  - [主页数据组件与标签规范设计规格](2026-10-18-homepage-components-design.md)
- 取代：`2026-10-17-subforum-custom-layout-design.md`（草案，不再实施）

## 1. 背景与目标

板块当前支持两种首页能力：

1. 默认模板：Hero（板块名、简介、成员/帖子数、加入/发帖/设置）+ 公告/版块/链接 + 帖子流。
2. 旧「自定义页面」：`custom_page_html` 经草稿 → 提交审核 → `APPROVED` 后，作为帖子流上方的内容块展示。

旧自定义页存在明显差距：iframe 仅 `sandbox="allow-popups"`，没有内联 CSP，没有受信任 shim、没有数据组件、没有大小与频率限制、没有举报与下架机制。

本需求参考个人自定义主页，为板块提供一套新的「自定义论坛首页」：

- **普通首页**：保持现有默认模板与帖子流，是默认值。
- **自定义首页**：版主/创建者上传一段 HTML，替换板块页左侧主内容区；右侧栏保留。

目标是在开放装修能力的同时保证：

1. 用户 HTML 不能执行任意脚本、不能读取登录态、不能访问本站后端。
2. 不能向第三方服务器发送信息或加载外部资源。
3. 只能通过受信任 shim 调用只读数据组件，组件数据经过服务端裁剪。
4. 外部链接必须弹出安全风险提示。
5. 自定义内容只能影响本板块首页，不影响站点其他页面。
6. 违规内容可被举报、管理员可下架与恢复。

## 2. 已确认决策

| 决策点 | 结论 |
| --- | --- |
| 替换范围 | 替换板块页左侧主内容区（Hero、门槛提示、排序栏、帖子列表）；保留站点导航与右侧栏 |
| 编辑权限 | 创建者 + OWNER/MODERATOR（沿用 `access::can_manage`）；全局管理员通过管理后台下架/恢复 |
| 审核方式 | 免审核，保存即生效；以「举报 + 管理员下架」兜底 |
| 数据组件 | 板块资料、板块统计、帖子列表（最新/热门）、成员列表 |
| 旧功能处理 | 新功能取代旧「自定义页面」；移除旧前端入口与 Rust 路由 |
| 旧数据迁移 | 不迁移；旧自定义页内容不再展示 |
| 发帖入口 | 自定义模式下不额外提供发帖入口；由版主在 HTML 中自行放置站内链接或依赖站内其他入口 |
| 规范文档 | 新增独立页面 `/settings/forum-homepage/guide` |
| 隔离方案 | 复用个人主页安全栈：净化器 + 沙箱 iframe + 受信任 shim + CSP |
| 样式载体 | 支持 `style` 属性；`<style>` 标签整段移除 |

## 3. 范围

### 3.1 包含

- 板块首页模式存储与切换（`NORMAL` / `CUSTOM`）。
- 自定义 HTML 的服务端严格净化、大小限制、保存频率限制。
- 自定义首页的公开读取、沙箱渲染与数据组件。
- 板块设置页「自定义首页」Tab（替换旧「自定义页面」Tab）。
- 板块页按模式渲染，自定义模式下保留右侧栏。
- 举报、管理员下架 / 恢复、管理后台列表。
- 独立装修规范页与完整示例。
- 移除旧自定义页的前端入口、渲染组件与 Rust 路由。

### 3.2 不包含

- 不执行用户上传的 JavaScript，也不提供脚本白名单。
- 不提供登录态透传、写操作、表单提交、评论/点赞组件。
- 不做发布前人工审核队列。
- 不支持 `<style>` / `<script>` / `<iframe>` / `<form>` / SVG / 外链资源。
- 不支持视频、音频、外部字体、外部图片、`data:` 资源。
- 不迁移旧 `custom_page_html` 数据，不删除旧数据库列。
- 不改变帖子、评论、个人主页与板块门槛的现有逻辑。
- 不新增版主任命 UI（本次仅让已有 OWNER/MODERATOR 能进入设置页）。

## 4. 架构总览

```
ForumPage.vue
  ├─ GET /api/forums/{slug}            → 板块详情（含 canManage，已过 Browse 门槛）
  ├─ GET /api/forums/{id}/homepage     → 首页模式与 HTML（已过 Browse 门槛）
  └─ NORMAL: 现有 Hero + 默认模板 + 帖子流
     CUSTOM: SafeHomepageFrame(html, loadData)
                ├─ sandbox="allow-scripts" iframe
                ├─ CSP default-src 'none' / connect-src 'none'
                ├─ 受信任 shim：高度、内部跳转、外链提示、data-request
                └─ 父页面收到 data-request 后调用
                     GET /api/forums/{id}/homepage-data
```

- 前端复用个人主页的 `SafeHomepageFrame.vue`，将其泛化为接收 `loadData` 函数，协议与 shim 共用。
- 后端复用 `sanitizer.rs` 与 `homepages.rs` 的实现模式，新增论坛主页路由模块，避免把论坛逻辑塞进个人主页模块。

## 5. 数据模型

启动迁移在 `backend-rs/src/migrate.rs` 中幂等创建两张表，不修改 `sub_forums` 主表结构。

### 5.1 `forum_homepages`

```sql
CREATE TABLE IF NOT EXISTS forum_homepages (
  forum_id BIGINT NOT NULL PRIMARY KEY,
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

| 字段 | 语义 |
| --- | --- |
| `mode` | `NORMAL` / `CUSTOM`，默认 `NORMAL` |
| `html` | 已净化的 HTML；切换回普通首页时保留，便于再次切回 |
| `status` | `ACTIVE` / `BLOCKED` |
| `block_reason` | 下架原因，仅管理者与管理员可见 |
| `size_bytes` | 净化后 HTML 的 UTF-8 字节数 |
| `updated_at` | 保存冷却判断与列表排序 |

无记录等价于「普通首页 + 无自定义内容」。

### 5.2 `forum_homepage_reports`

```sql
CREATE TABLE IF NOT EXISTS forum_homepage_reports (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  forum_id BIGINT NOT NULL,
  reporter_id BIGINT NOT NULL,
  reason VARCHAR(500) NOT NULL,
  created_at DATETIME NOT NULL,
  KEY idx_target_time (forum_id, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

### 5.3 迁移行为

- 只建表，不迁移旧 `custom_page_html`。
- 旧列 `custom_page_html`、`custom_page_status`、`custom_page_review_note` 保留在数据库中，但代码不再读写。
- 旧 `template_type='CUSTOM'` 的板块在新逻辑下按 `NORMAL` 处理（无 `forum_homepages` 记录），展示默认模板。

## 6. API 设计

所有接口沿用 `{code, message, data}` 响应包裹与可选登录提取器 `CurrentUser`。

通用规则：

- 板块不存在 → 404「子论坛不存在」。
- 读取类接口先执行 `gate::ensure(pool, viewer_id, is_admin, &forum, GateKind::Browse)`。
- 管理类接口先执行 `access::can_manage(pool, &forum, Some(me.id))`，失败返回 403「需要坛主或管理员权限」。
- 公开字段只包含板块公开信息与用户名/头像/角色，不返回邮箱、认证、用户信息项。

### 6.1 公开读取首页配置

`GET /forums/{id}/homepage`

- 无需登录；执行 Browse 门槛。
- 返回分支：

| 条件 | 返回 |
| --- | --- |
| 无记录或 `mode=NORMAL` | `{ "mode": "NORMAL" }` |
| `mode=CUSTOM` 且 `status=ACTIVE` | `{ "mode": "CUSTOM", "html": "...", "blocked": false, "updatedAt": "..." }` |
| `mode=CUSTOM` 且 `status=BLOCKED`，请求者为管理者或管理员 | `{ "mode": "CUSTOM", "html": "...", "blocked": true, "blockReason": "...", "updatedAt": "..." }` |
| `mode=CUSTOM` 且 `status=BLOCKED`，其他访问者 | `{ "mode": "NORMAL" }` |

「管理者」判定：`access::can_manage`；「管理员」判定：`is_admin()`。

### 6.2 管理者读取完整配置

`GET /forums/{id}/homepage/mine`

- 需登录且为管理者。
- 返回：

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

无记录时返回 `mode=NORMAL`、`html=null`、`status=ACTIVE`、`sizeBytes=0`。

### 6.3 保存首页

`PUT /forums/{id}/homepage`

请求：

```json
{ "mode": "NORMAL" | "CUSTOM", "html": "<div>...</div>" }
```

规则：

- `mode` 只能是 `NORMAL` / `CUSTOM`，否则 400。
- `NORMAL`：只切换模式，保留已有 HTML 与下架状态。
- `CUSTOM`：
  - HTML 去空白后不能为空，否则 400。
  - 原始 HTML ≤ 64KB，否则 400。
  - 使用 `sanitizer::sanitize_forum_homepage` 净化；净化后为空 → 400。
  - 净化后 ≤ 128KB，否则 400。
  - 保存后不改变 `status`：已被下架的首页修改内容后仍需管理员恢复。
- 冷却：同一板块距上次保存 < 3 秒 → 429「保存过于频繁，请稍后再试」。
- 返回同 6.2 的完整配置。

### 6.4 重置普通首页

`POST /forums/{id}/homepage/reset`

- 管理者才能调用。
- 事务内删除 `forum_homepage_reports` 中该板块记录与 `forum_homepages` 记录。
- 返回 `mode=NORMAL` 的空配置。

### 6.5 举报

`POST /forums/{id}/homepage/report`

请求：

```json
{ "reason": "5-500 个字符" }
```

规则：

- 需登录。
- 不能举报自己管理的板块（`access::can_manage` 为真）→ 400「不能举报自己管理的社区首页」。
- 目标必须存在且 `mode=CUSTOM`、`status=ACTIVE`，否则 400「该社区没有可举报的自定义首页」。
- 原因按字符数校验 5–500。
- 同一用户对同一板块 24 小时内只能举报一次，否则 400「你已举报过该首页」。
- 成功返回 `{ "ok": true }`。

### 6.6 数据组件数据

`GET /forums/{id}/homepage-data`

- 无需登录；执行 Browse 门槛。
- 返回：

```json
{
  "forum": {
    "id": 12,
    "slug": "rust",
    "name": "Rust 社区",
    "description": "……",
    "icon": "/forum/api/media/9",
    "themeColor": "#4f46e5",
    "createdAt": "2026-10-18T12:00:00"
  },
  "stats": { "members": 128, "posts": 64, "comments": 320 },
  "posts": [
    {
      "id": 1001,
      "title": "帖子标题",
      "score": 8,
      "commentCount": 3,
      "authorName": "alice",
      "createdAt": "2026-10-18T11:00:00"
    }
  ],
  "hotPosts": [],
  "members": [
    {
      "id": 5,
      "username": "alice",
      "avatar": "/forum/api/media/3",
      "role": "OWNER",
      "joinedAt": "2026-10-01T09:00:00"
    }
  ]
}
```

查询口径：

- `stats.members`：`forum_members` 中该板块记录数。
- `stats.posts`：该板块 `posts.is_removed = 0` 的数量。
- `stats.comments`：该板块未删除帖子下的未删除评论数。
- `posts`：该板块 `is_removed = 0` 的帖子，`created_at DESC, id DESC`，最多 10 条。
- `hotPosts`：同范围，`score DESC, comment_count DESC, created_at DESC, id DESC`，最多 10 条。
- `members`：`forum_members JOIN users`，按 `OWNER → MODERATOR → MEMBER`、`joined_at ASC` 排序，最多 10 条。
- `description` 截断到 200 字符；`icon`/`avatar` 由前端 shim 再校验为 `/forum/api/media/{数字id}`。
- 所有列表都是只读快照，不返回帖子正文、评论正文、邮箱等敏感字段。

### 6.7 管理端

- `GET /api/admin/forum-homepages`
  - 仅管理员。
  - 返回 `mode=CUSTOM` 的板块列表：`forumId, forumName, forumSlug, creatorName, status, blockReason, sizeBytes, reportCount, updatedAt`。
  - 按 `updated_at DESC` 排序，最多 100 条。
- `POST /api/admin/forum-homepages/{forumId}/block`
  - 请求 `{ "reason": "..." }`，原因 5–500 字。
  - 设置 `status=BLOCKED`、`block_reason`、`blocked_by`、`blocked_at`。
- `POST /api/admin/forum-homepages/{forumId}/unblock`
  - 设置 `status=ACTIVE`，清空下架字段。

### 6.8 板块详情补充字段

`GET /forums/{slug}` 的 `data` 增加：

```json
{ "canManage": true }
```

- 计算方式：`access::can_manage(pool, &forum, viewer_id)`；匿名用户为 `false`。
- 仅用于前端显示「管理社区」入口与允许进入设置页；实际写操作仍由后端接口校验。
- 移除 `customPageHtml/customPageStatus/customPageReviewNote` 输出。

## 7. 净化器与组件

### 7.1 净化器拆分

`backend-rs/src/sanitizer.rs` 拆为两个入口，共用基础标签、属性、CSS 与 URL 规则：

```rust
pub fn sanitize_user_homepage(html: &str) -> String
pub fn sanitize_forum_homepage(html: &str) -> String
```

现有 `sanitize_homepage` 保留为 `sanitize_user_homepage` 的薄封装，避免无谓改动调用方与既有测试。

- 基础标签：沿用现有个人主页白名单（`a/div/span/p/h1-h6/table/img/...`）。
- 个人主页额外允许：`of-profile`、`of-stats`、`of-posts`。
- 论坛首页额外允许：`of-forum`、`of-forum-stats`、`of-forum-posts`、`of-members`。
- 两类页面互相不包含对方的组件标签；被移除的标签保留内部文本（ammonia 行为），文档需说明。
- `script/style/iframe/object/embed/svg/math/form/input/button/select/textarea/video/audio/canvas/meta/link/base` 等仍列入 `clean_content_tags`，连同内容一起移除。
- `style` 属性继续按 `css_properties()` 白名单逐条过滤；`url()`、`@import`、`expression()`、反斜杠等仍拒绝。
- `img src` 只允许 `/forum/api/media/{数字id}`；`a href` 只允许 `http/https/mailto`、单斜杠站内路径与页内锚点。

### 7.2 组件属性

| 组件 | 属性 | 规则 |
| --- | --- | --- |
| `of-forum` | `heading` | 1–30 字符，去除首尾空白，拒绝控制字符 |
| `of-forum-stats` | `heading` | 同上 |
| `of-forum-posts` | `type` | `latest` / `hot`；非法值移除属性，前端回退 `latest` |
| `of-forum-posts` | `limit` | 1–10 的十进制整数；非法值移除属性，前端回退 5 |
| `of-forum-posts` | `heading` | 1–30 字符 |
| `of-members` | `limit` | 1–10 的十进制整数；非法值移除属性，前端回退 5 |
| `of-members` | `heading` | 1–30 字符 |

所有 `on*` 事件属性、`class`/`id`/`style` 之外的未知属性仍由白名单剥离。

### 7.3 单页限制

- 每个页面最多渲染 20 个数据组件（shim 取文档顺序前 20 个）。
- 每个列表最多 10 条；`limit` 只控制展示条数，不改变服务端返回条数。
- 自定义首页 HTML 原始上限 64KB，净化后上限 128KB。

## 8. 前端设计

### 8.1 沙箱框架泛化

修改 `frontend/src/components/SafeHomepageFrame.vue`：

- Props 改为：

```ts
defineProps<{
  html: string
  loadData?: () => Promise<unknown>
  title?: string
}>()
```

- 删除对 `homepageApi` 与 `HomepageData` 的直接依赖；`provideData` 调用 `props.loadData()`。
- 保留现有安全实现：
  - `sandbox="allow-scripts"`、`referrerpolicy="no-referrer"`、`scrolling="no"`。
  - CSP：`default-src 'none'`、`img-src ${origin}/forum/api/media/`、`style-src 'unsafe-inline'`、`script-src 'nonce-...'`、`connect-src 'none'`、`font-src/media-src/object-src/frame-src/worker-src/form-action/base-uri` 全部关闭。
  - nonce 握手、frame 身份校验、只响应一次 data-request、高度 480–4000px。
  - 内部链接交给 `vue-router`；外部链接弹「安全风险提示」确认后 `noopener,noreferrer` 打开。
- `Profile.vue` 改为 `:load-data="loadHomepageData"`；行为不变。

### 8.2 shim 扩展

在 shim 中新增组件查询与渲染：

```js
document.querySelectorAll(
  'of-profile, of-stats, of-posts, of-forum, of-forum-stats, of-forum-posts, of-members'
)
```

- `of-forum`：头像/图标、名称、`f/{slug}`、简介（最多 200 字）、创建时间。
- `of-forum-stats`：成员、帖子、评论三个统计卡。
- `of-forum-posts`：`type=hot` 使用 `data.hotPosts`，否则使用 `data.posts`；链接 `/post/{id}`；元信息为「作者 · 赞 · 评论 · 日期」。
- `of-members`：头像、用户名、角色（坛主/版主/成员）、加入时间；链接 `/u/{id}`。
- 空态文案：`暂无帖子`、`暂无成员`。
- `icon` 只接受 `/forum/api/media/{数字id}`，否则显示名称首字；`themeColor` 只接受 `#RGB/#RRGGBB/#RRGGBBAA`，否则忽略。
- 所有文本通过 `textContent` 写入；所有链接由数字 ID 拼接，不直接使用服务端字符串作为 `href`。

### 8.3 板块页

`frontend/src/views/ForumPage.vue`：

- 新增状态：`homepageMode`、`homepageHtml`、`homepageBlocked`、`homepageBlockReason`。
- `loadForum()` 在板块详情成功后调用 `forumApi.homepage(forum.id)`：
  - 失败时按 `NORMAL` 处理，不阻塞页面。
  - `CUSTOM` 时不加载默认帖子流。
- 模板结构调整：

```vue
<template v-if="customHomeActive">
  <el-alert v-if="homepageBlocked" type="warning" ...>
    该自定义首页已被下架，访客当前看到默认首页。原因：{{ homepageBlockReason }}
  </el-alert>
  <SafeHomepageFrame
    :html="homepageHtml"
    :load-data="loadForumHomepageData"
    title="社区自定义首页"
  />
</template>
<template v-else>
  <!-- 现有 Hero、门槛提示、默认模板、排序栏、帖子列表 -->
</template>
```

- 右侧栏保留，并新增：
  - 管理者（`forum.canManage`）显示「管理社区」按钮，跳转 `/f/{slug}/settings`。
  - `CUSTOM` 且已登录且非管理者时显示「举报该首页」文字按钮，打开举报弹窗。
- 举报弹窗：原因 5–500 字，提交 `forumApi.reportHomepage(forum.id, reason)`，成功后提示「举报已提交」。
- 自定义模式下不加载默认帖子流，但保留发帖弹窗逻辑（不提供额外入口）。
- 若 `mode=CUSTOM` 但 HTML 为空或净化结果为空，按 `NORMAL` 渲染，避免空白页。

### 8.4 板块设置页

`frontend/src/views/ForumSettings.vue`：

- 权限判断从 `isOwner` 改为 `forum.canManage`。
- 移除「页面模板」单选项（旧 `template_type` 语义）与默认模板 Tab 的相关提示；`templateType` 仍按原值提交，避免影响后端。
- 将「自定义页面」Tab 替换为「自定义首页」：
  - 模式单选：普通首页 / 自定义首页。
  - HTML 文本域（16 行），显示字符数与 64KB 限制。
  - 工具栏：插入图片、插入完整示例、组件文档、清空。
  - 保存按钮：`forumApi.saveHomepage(forum.id, { mode, html })`。
  - 重置按钮：确认后 `forumApi.resetHomepage(forum.id)`。
  - 下架提示：`status=BLOCKED` 时显示原因与「访客当前看到默认首页」。
  - 「打开装修规范」链接到 `/settings/forum-homepage/guide`。
- 插入图片沿用个人主页装修页逻辑：上传到 `/media`，成功后插入 `<img src="/forum/api/media/{id}" alt="">`。

### 8.5 规范文档页

新增 `frontend/src/views/ForumHomepageGuide.vue`，路由 `/settings/forum-homepage/guide`（需登录）。

内容：

1. 快速开始：设置 → 自定义首页 → 粘贴 HTML → 保存。
2. 可用标签：基础标签列表。
3. 不可用标签：`script/style/iframe/object/embed/form/input/button/svg/math/video/audio/canvas/meta/link/base` 等，并单独说明 `of-profile/of-stats/of-posts` 仅用于个人主页，论坛首页会被移除。
4. 属性规则、CSS 白名单、图片与链接规则。
5. 论坛组件参数表（四个组件及属性）。
6. 完整示例 + 复制按钮。
7. FAQ：模式区别、下架后行为、门槛关系、非法 `type`/`limit` 的回退、外链提示、图片上传限制。

示例 HTML 覆盖：板块资料、统计、最新帖子、热门帖子、成员列表，以及站内链接与外链示例。

### 8.6 管理后台

`frontend/src/views/Admin.vue` 新增「论坛首页」Tab：

- 表格列：板块、坛主、状态、举报数、更新时间、操作。
- 操作：下架（填写原因）、恢复。
- 移除旧「自定义页面」审核 Tab 与相关逻辑。

### 8.7 前端 API 与类型

`frontend/src/api/index.ts`：

```ts
forumApi.homepage(id)
forumApi.myHomepage(id)
forumApi.saveHomepage(id, { mode, html })
forumApi.resetHomepage(id)
forumApi.reportHomepage(id, reason)
forumApi.homepageData(id)
adminApi.forumHomepages()
adminApi.blockForumHomepage(id, reason)
adminApi.unblockForumHomepage(id)
```

移除旧 `getCustomPage / saveCustomPage / submitCustomPage` 与管理员自定义页方法。

`frontend/src/types/index.ts` 新增：

- `ForumHomepageMode`
- `PublicForumHomepage`
- `MyForumHomepage`
- `ForumHomepageData`（`forum/stats/posts/hotPosts/members`）
- `ForumHomepagePost`
- `ForumMemberSummary`
- `AdminForumHomepageItem`

并在 `SubForum` 增加 `canManage?: boolean`；移除 `customPageHtml/customPageStatus/customPageReviewNote` 的前端使用（类型字段可保留以兼容旧响应，但页面不再读取）。

## 9. 安全模型

| 威胁 | 防护 |
| --- | --- |
| XSS / 事件处理器 | ammonia 白名单 + 属性白名单 + 禁用 `script/style/iframe/svg` 等；`on*` 全部剥离 |
| CSS 注入 / 外链请求 | CSS 属性白名单；拒绝 `url()`、`@import`、`expression()`、反斜杠 |
| 外部图片 / 追踪像素 | `img src` 只允许 `/forum/api/media/{数字id}`；CSP `img-src` 仅站内媒体 |
| 向第三方发请求 | `sandbox` 无 `allow-same-origin`；CSP `connect-src 'none'`、`default-src 'none'` |
| 读取登录态 / 父页面数据 | 跨域沙箱 iframe，无法访问父页面 DOM 与存储；消息必须匹配 nonce 与 frame 身份 |
| 绕过门槛读取板块数据 | `/homepage` 与 `/homepage-data` 均执行 Browse 门槛；未过门槛不返回 HTML 或数据 |
| 越权编辑 | 保存/重置/读取完整配置均校验 `access::can_manage` |
| 数据泄漏 | 组件数据只含公开字段；不返回邮箱、认证、用户信息项、帖子/评论正文 |
| 违规内容 | 登录用户举报（24 小时去重）；管理员下架；下架内容对访客完全隐藏并回退默认首页 |
| 资源耗尽 | 原始 64KB / 净化后 128KB / 3 秒冷却 / 单页 20 组件 / 列表 10 条 |
| 外链钓鱼 | 外链点击弹出域名与风险提示，确认后 `noopener,noreferrer` 新窗口打开 |
| 表单提交 / 导航劫持 | shim 拦截 `submit`；内部路径只允许 `/` 开头且非 `//`；外部仅 `http(s)` |

## 10. 旧自定义页面移除

### 10.1 前端

- `ForumSettings.vue`：删除「自定义页面」Tab、`customHtml/customStatus/customReviewNote` 状态与保存/提交函数。
- `ForumPage.vue`：删除 `CustomPageRenderer` 引用、`customHtml` 与 `getCustomPage` 回退加载。
- 删除 `frontend/src/components/CustomPageRenderer.vue`。
- `api/index.ts`：删除旧自定义页与管理员审核方法。
- `Admin.vue`：删除旧自定义页审核 Tab。
- `router/index.ts`：删除旧自定义页路由（如有）。

### 10.2 后端

- `routes/mod.rs`：删除 `/forums/:id/custom-page`、`/forums/:id/custom-page/submit`、`/admin/custom-pages*`。
- `routes/forums.rs`：删除 `CustomPageUpdateRequest`、`custom_page_dto`、`save_custom_page`、`submit_custom_page`、`get_custom_page`。
- `routes/admin.rs`：删除 `list_custom_pages`、`approve_custom_page`、`reject_custom_page` 及其请求结构。
- `build_detail`：不再输出 `customPageHtml/customPageStatus/customPageReviewNote`。
- `sanitizer::sanitize`（旧 jsoup 风格宽松净化）如无其他调用则删除。

### 10.3 数据库

- 保留 `sub_forums.custom_page_html/custom_page_status/custom_page_review_note` 列，不执行 DROP。
- 不读取、不写入、不迁移这些列。

## 11. 规范文档内容要求

`/settings/forum-homepage/guide` 必须明确写出：

- 哪些标签可用、哪些不可用（逐项列表 + 原因）。
- 个人主页专用组件在论坛首页不可用。
- 可用属性与 CSS 属性白名单。
- 图片只能使用站内上传地址。
- 外链会弹风险提示；表单、脚本、iframe 不可用。
- 四个组件的参数、默认值与上限。
- 完整可复制示例。
- 非法 `type` / `limit` 会被移除并按默认值展示。
- 下架状态、举报机制与保存频率限制。

## 12. 测试与验收

### 12.1 单元测试

- `sanitize_forum_homepage`：
  - 保留 `of-forum/of-forum-stats/of-forum-posts/of-members` 与合法属性。
  - 移除 `of-profile/of-stats/of-posts` 标签（保留内部文本）。
  - 非法 `type` / `limit` / 超长 `heading` 移除属性。
  - 脚本、iframe、表单、事件处理器、外链图片、`javascript:` 链接被移除。
- `sanitize_user_homepage` 行为不回归（现有 19 项测试继续通过）。

### 12.2 本地 API E2E

- 创建用户 A（创建者）、用户 B（普通访客）、用户 C（管理员可选）。
- A 创建板块与 12 篇帖子，B 发 1 篇，创建评论与点赞。
- 保存自定义首页（含四个组件、非法标签、外链图片、脚本、事件处理器）。
- 断言：
  - 保存后 HTML 已净化；组件与合法属性保留；非法属性移除。
  - 公开读取返回 `mode=CUSTOM` 与净化后的 HTML。
  - 匿名与 B 请求 `homepage-data`：`stats` 与列表正确，最多 10 条，热门排序正确，成员仅公开字段。
  - 设置 Browse 门槛后，未满足者请求 `/homepage` 与 `/homepage-data` 均 403。
  - 举报：5 字以下拒绝、24 小时重复拒绝、正常举报成功。
  - 管理员下架后：访客读到 `mode=NORMAL`；管理者读到 `blocked=true` 与原因；恢复后访客恢复 `CUSTOM`。
  - 保存冷却：连续两次保存第二次 429。
  - 重置后记录与举报清空，返回 `NORMAL`。
- 旧自定义页路由返回 404/不存在（确认已移除）。

### 12.3 构建检查

- `cargo test`、`cargo clippy --all-targets`。
- `npm run type-check`、`npm run build`。

### 12.4 浏览器验收

- 普通首页与自定义首页切换正确；右侧栏始终存在。
- 四个组件渲染正确，最新/热门帖子顺序正确，成员角色显示正确。
- 内部帖子/用户链接在父页面路由跳转。
- 外部链接弹出安全提示；取消不跳转，确认后新标签打开。
- 页面无任何外部网络请求；无脚本执行痕迹。
- 被下架时访客看到默认首页，管理者看到提示与自定义内容。
- 设置页保存/重置/下架提示正常；规范页各章节与复制按钮正常。
- 管理后台可下架/恢复论坛首页。

### 12.5 生产烟测

- 部署后端与前端后，创建临时板块与自定义首页，验证：
  - `GET /forum/api/forums/{id}/homepage` 字段与门槛。
  - `GET /forum/api/forums/{id}/homepage-data` 数据形状与公开字段。
  - 页面渲染、内部跳转、无外部请求。
- 清理临时数据。

## 13. 发布与回滚

- 后端先部署：新增表与接口，旧前端不受影响。
- 前端随后部署：切换到新模式并移除旧入口。
- 回滚策略：
  - 前端回滚到上一版本即可恢复旧自定义页展示（旧数据未删除）。
  - 后端回滚保留 `forum_homepages` 表，不影响既有功能。
  - 不回滚用户已保存的新首页数据；如需紧急止损，管理员可将相关板块首页下架或前端回滚。

## 14. 风险与取舍

| 风险 | 说明 | 处理 |
| --- | --- | --- |
| 免审核导致违规内容短时可见 | 与个人主页一致 | 举报 + 管理员下架；管理后台列表按更新时间排序 |
| 自定义模式隐藏发帖按钮 | 用户选择不额外提供入口 | 文档说明可在 HTML 中放置站内链接；默认模式仍保留 |
| 旧自定义页内容突然下线 | 不迁移旧数据 | 已与用户确认；发布说明中提示版主重新装修 |
| iframe 内容不被搜索引擎索引 | 沙箱与 CSP 的必然结果 | 接受；板块详情 API 与默认模式仍可收录 |
| 组件数据被用于推断 | 仅公开统计与标题 | 数据经过裁剪；不提供评论正文、成员邮箱等 |
| 版主任命 UI 缺失 | 本次不涉及 | 后端已支持 OWNER/MODERATOR；已有角色可管理，后续单独补 UI |
