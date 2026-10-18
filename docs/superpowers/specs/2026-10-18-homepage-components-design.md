# 自定义个人主页数据组件与标签规范设计

- 日期：2026-10-18
- 状态：待评审
- 范围：Rust 后端（`backend-rs`）+ Vue 前端（`frontend`）
- 关联规格：[自定义个人主页设计](2026-10-18-user-homepage-design.md)

## 1. 背景与目标

自定义个人主页目前只支持纯静态 HTML：用户无法展示自己的公开资料、帖子、点赞与评论记录，只能手写固定文案，内容很快过期。

本需求在不放宽现有安全边界的前提下，为自定义主页增加**声明式数据组件**，并提供一份面向用户的**标签规范文档**与**完整示例代码**。

目标：

1. 用户可以在 HTML 中声明 `<of-profile>`、`<of-stats>`、`<of-posts>` 组件，展示个人信息与帖子信息。
2. 数据由父页面代为请求，iframe 内不直接访问后端，`connect-src 'none'` 保持不变。
3. 只返回公开资料与公开统计，**不返回邮箱、认证信息、用户信息项**。
4. 帖子 / 点赞 / 评论列表遵守板块浏览门槛，查看者无权浏览的内容不出现。
5. 提供 `/settings/homepage/guide` 规范文档页，明确「哪些标签能用、哪些不能用」。
6. 装修页提供「组件文档」与「插入完整示例」入口，示例覆盖个人信息、帖子、点赞、评论。
7. 不引入新的脚本执行能力、外部请求能力或表单提交能力。

## 2. 已确认决策

| 决策点 | 结论 |
| --- | --- |
| 「个人信息」范围 | 仅公开资料（用户名、头像、简介、角色、加入时间）+ 统计（帖子数、评论数、获赞数） |
| 信息项（性别/年龄/城市等） | 不返回，不下发到 iframe |
| 点赞 / 评论 | 只读列表，不允许在主页内发起点赞、评论等写操作 |
| 数据获取方式 | 方案 A：声明式组件 + 父页面代理取数 |
| 沙箱策略 | 不变：`sandbox="allow-scripts"`，无 `allow-same-origin` |
| CSP | 不变：`default-src 'none'`、`connect-src 'none'` |
| 组件数量 | 单页最多 20 个组件实例，所有组件合并为一次取数 |
| 单列表条数 | 每个列表最多 10 条，默认 5 条 |
| 失败表现 | 组件内显示「内容加载失败，请刷新重试」，不自动重试 |

## 3. 范围

### 3.1 包含

- 净化器新增三个组件标签及其受控属性。
- 新增 `GET /api/users/{id}/homepage-data` 只读接口。
- `SafeHomepageFrame` 扩展消息协议：`data-request` / `data-response`。
- shim 内组件渲染（全部使用 `textContent` / `createElement`）。
- 装修页新增组件文档入口与完整示例。
- 新增 `/settings/homepage/guide` 规范文档页。
- 净化器单测、接口测试、类型检查、构建与线上浏览器验证。

### 3.2 不包含

- 不执行用户 JavaScript，不新增脚本白名单。
- 不提供点赞、评论、关注等写操作。
- 不返回邮箱、认证徽章、认证等级、用户信息项。
- 不返回帖子正文，只返回标题与摘要级元数据。
- 不支持自定义数据源、外部 API、RSS、外链图片。
- 不改变普通个人主页与帖子/评论页的现有渲染。
- 不新增数据库表或字段。

## 4. 组件语法

### 4.1 总览

| 标签 | 作用 | 可用属性 |
| --- | --- | --- |
| `<of-profile>` | 展示主页所属用户的公开资料 | `heading` + 全局属性 |
| `<of-stats>` | 展示帖子数、评论数、获赞数 | `heading` + 全局属性 |
| `<of-posts>` | 展示帖子列表 | `type`、`limit`、`heading` + 全局属性 |

全局属性沿用现有白名单：`class`、`id`、`title`、`dir`、`lang`、`style`。

组件标签名与属性名不区分大小写；`type` 的值统一按小写处理。

### 4.2 `<of-profile>`

```html
<of-profile heading="关于我"></of-profile>
```

渲染内容：

- 头像：仅当值为 `/forum/api/media/{纯数字}` 时显示，否则显示用户名首字符占位块。
- 用户名。
- 角色：`ADMIN` → 「管理员」，其余 → 「用户」。
- 简介：纯文本，保留换行，最多展示 200 字（服务端已截断）。
- 加入时间：`YYYY-MM-DD`。

字段缺失时对应行不渲染，不显示 `null` 或空白占位。

### 4.3 `<of-stats>`

```html
<of-stats heading="社区数据"></of-stats>
```

渲染三个统计块：

| 键 | 展示名 | 语义 |
| --- | --- | --- |
| `posts` | 帖子 | 未被删除的帖子总数 |
| `comments` | 评论 | 未被删除的评论总数 |
| `upvotes` | 获赞 | 上述帖子与评论收到的赞成票总和 |

统计为全站聚合值，不受查看者板块门槛影响；统计不包含任何板块名或内容。

### 4.4 `<of-posts>`

```html
<of-posts type="authored" limit="5" heading="我的帖子"></of-posts>
<of-posts type="liked" limit="5" heading="我点赞的帖子"></of-posts>
<of-posts type="commented" limit="5" heading="我评论过的帖子"></of-posts>
```

| 属性 | 取值 | 默认 | 说明 |
| --- | --- | --- | --- |
| `type` | `authored` / `liked` / `commented` | `authored` | 我发布的 / 我点赞的 / 我评论过的 |
| `limit` | 1–10 的整数 | `5` | 展示条数 |
| `heading` | ≤ 30 字 | 无 | 列表标题；空值或超长时移除 |

非法 `type`、非法 `limit` 在服务端净化阶段被移除，前端按默认值渲染。客户端渲染前再次校验，非法值一律回退默认。

渲染结构：

- 可选标题。
- 每条：帖子标题（链接到 `/forum/post/{id}`）、板块名、分数、评论数、发布时间。
- 空列表：「暂无帖子」/「暂无点赞」/「暂无评论」。
- 仅使用响应中的数字 `id` 拼接链接，不使用标题或板块 slug 参与 URL 构造。

### 4.5 位置与数量

- 组件可放在任意允许的块级容器内。
- 单页最多处理 20 个组件实例；超出部分保持占位文案，不填充数据。
- 所有组件共用一次数据请求，按 `type` 从同一份响应中取数组并按 `limit` 截断。
- 组件可嵌套普通标签；组件内部原有内容视为加载中占位，数据到达后被替换。

### 4.6 占位内容

组件内部文字在数据到达前展示：

```html
<of-posts type="authored">正在加载我的帖子…</of-posts>
```

未填写占位内容时，shim 默认显示「加载中…」。请求失败时统一替换为「内容加载失败，请刷新重试」。

## 5. 数据接口

### 5.1 路由与鉴权

```text
GET /api/users/{id}/homepage-data
```

- 公开接口，无需登录；登录用户携带 JWT 时按该用户计算板块浏览权限。
- 用户不存在 → `404`。
- 只读，无请求体。
- 响应 `Content-Type: application/json`。

### 5.2 响应结构

```json
{
  "profile": {
    "id": 14,
    "username": "alice",
    "avatar": "/forum/api/media/12",
    "bio": "热爱分享",
    "role": "USER",
    "createdAt": "2026-01-02T03:04:05"
  },
  "stats": { "posts": 12, "comments": 34, "upvotes": 56 },
  "posts": [
    {
      "id": 101,
      "title": "一篇帖子",
      "forumName": "综合讨论",
      "forumSlug": "general",
      "score": 7,
      "commentCount": 3,
      "createdAt": "2026-10-01T12:00:00"
    }
  ],
  "liked": [],
  "commented": []
}
```

### 5.3 字段约束

`profile` 只包含以下字段，**不得包含** `email`、`authentications`、认证等级、`user_info` 信息项：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | number | 用户 ID |
| `username` | string | 用户名 |
| `avatar` | string \| null | 站内头像地址 |
| `bio` | string \| null | 简介，最多 200 字 |
| `role` | string | `ADMIN` / `USER` |
| `createdAt` | string | 注册时间 |

`posts` / `liked` / `commented` 每项字段一致：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `id` | number | 帖子 ID |
| `title` | string | 标题 |
| `forumName` | string | 板块名 |
| `forumSlug` | string | 板块 slug，仅用于展示 |
| `score` | number | 当前分数 |
| `commentCount` | number | 评论数 |
| `createdAt` | string | 发布时间 |

### 5.4 查询与过滤规则

| 列表 | 数据来源 | 排序 | 过滤 |
| --- | --- | --- | --- |
| `posts` | `posts.author_id = {id}` | `created_at DESC` | `is_removed = 0` |
| `liked` | `votes` 中 `user_id = {id}`、`target_type = 'POST'`、`value = 1` 关联 `posts` | `votes.updated_at DESC` | 帖子 `is_removed = 0`，同一帖子去重 |
| `commented` | `comments.author_id = {id}` 关联 `posts` | 该用户最近一条评论时间 `DESC` | 评论与帖子均 `is_removed = 0`，同一帖子去重 |

三个列表都必须通过 `gate::allowed_forum_ids(pool, viewer_id, is_admin)` 过滤，只保留查看者有权浏览的板块。先过滤、后截断，确保不会因为被过滤项占位而少于 `limit`。

每个列表固定返回最多 10 条，前端按组件 `limit` 截断。

`stats` 使用聚合查询：帖子数、评论数、获赞数均基于未删除内容，不按查看者门槛变化。

## 6. 安全与渲染

### 6.1 服务端净化扩展

在 `sanitizer::sanitize_homepage` 现有白名单基础上：

1. 标签白名单新增 `of-profile`、`of-stats`、`of-posts`。
2. `of-profile` / `of-stats` 允许 `heading`。
3. `of-posts` 允许 `type`、`limit`、`heading`。
4. `attribute_filter` 校验：
   - `type`：小写后必须为 `authored` / `liked` / `commented`，否则移除。
   - `limit`：去除空白后必须是 1–10 的十进制整数，否则移除。
   - `heading`：去除首尾空白，字符数 1–30，含控制字符则移除。
5. 未知 `of-*` 标签不在白名单内，按普通未知标签处理：标签被移除，内部文字保留为纯文本。
6. 组件标签上的 `on*`、`href`、`src` 等非白名单属性一律移除。

净化器继续使用现有 `clean_content_tags`、URL 协议、图片地址与 CSS 白名单，不新增任何脚本、样式表或外链资源通道。

### 6.2 消息协议

协议继续以 `__openforum_homepage: 1` 作为命名空间，新增两类消息：

iframe → 父页面：

```json
{ "__openforum_homepage": 1, "type": "data-request", "nonce": "<16字节hex>" }
```

父页面 → iframe：

```json
{ "__openforum_homepage": 1, "type": "data-response", "nonce": "<同一nonce>", "ok": true, "data": { } }
```

失败时 `ok: false`，不带 `data`。

父页面校验：

- `event.source === frame.contentWindow`。
- `data.__openforum_homepage === 1`。
- `data.type === 'data-request'`。
- `data.nonce` 与当前 frame 注入的 nonce 完全一致。
- 每个 frame 文档只响应一次取数请求，忽略重复请求。

`html` prop 变化时视为新文档：重新生成 nonce、重置「已响应」状态与高度，旧文档的迟到消息因 nonce 不匹配被忽略。

iframe 校验：

- `event.source === window.parent`。
- `data.__openforum_homepage === 1`。
- `data.type === 'data-response'`。
- `data.nonce === NONCE`。
- 同一文档只接受第一条响应。

父页面向 iframe 回传时，由于 sandbox 文档为不透明来源，`targetOrigin` 使用 `'*'`；安全性由「frame 身份 + 一次性 nonce + 仅公开数据」保证。

### 6.3 父页面取数

- `SafeHomepageFrame` 新增必填 prop `userId: number`。
- shim 仅在页面存在至少一个组件时发送 `data-request`；无组件时父页面不发请求。
- 收到 `data-request` 后调用一次 `GET /forum/api/users/{userId}/homepage-data`。
- 请求 URL 由数字 `userId` 拼接，不接受 iframe 传入的任何路径或参数。
- 请求失败、超时、非 2xx 时回传 `ok: false`，不重试。
- 组件卸载后忽略迟到的响应。

### 6.4 iframe 内渲染

shim 负责把数据变成 DOM，规则：

1. 只使用 `document.createElement`、`textContent`、`setAttribute`，禁止 `innerHTML`、`insertAdjacentHTML`、`eval`、`Function`。
2. 组件查找使用 `document.querySelectorAll('of-profile, of-stats, of-posts')`，最多处理前 20 个。
3. 读取 `type` / `limit` / `heading` 时再次校验，非法值回退默认。
4. 头像 `src` 仅接受 `/^\/forum\/api\/media\/\d+$/`，否则不创建 `<img>`。
5. 帖子链接仅由数字 ID 构造：`/forum/post/{id}`；点击继续走现有 `internal` 消息与父页面路由。
6. 标题、板块名、用户名、简介、标题文字全部用 `textContent` 写入，不做 HTML 拼接。
7. 组件样式来自受信任的 shim 内联样式表（`.of-*` 类），用户可在外层标签通过 `style` 覆盖。
8. 渲染完成后主动上报一次高度，`ResizeObserver` 继续负责后续变化。

### 6.5 现有防线不变

| 防线 | 状态 |
| --- | --- |
| 64KB 输入上限 / 净化后 128KB 上限 | 不变 |
| 3 秒保存冷却 | 不变 |
| ammonia 白名单 + CSS 白名单 | 仅新增三个组件标签与受控属性 |
| `sandbox="allow-scripts"`（无 `allow-same-origin`） | 不变 |
| CSP `default-src 'none'` / `connect-src 'none'` | 不变 |
| 外链确认弹窗、站内路由白名单 | 不变 |
| 举报 + 管理员下架 | 不变 |

### 6.6 威胁分析

| 威胁 | 处理 |
| --- | --- |
| 通过组件属性注入脚本 | 属性白名单 + 值校验；`on*` 全部移除 |
| 通过标题/简介注入 HTML | 全程 `textContent` |
| 伪造 `data-response` 注入数据 | 校验 `event.source`、命名空间、一次性 nonce |
| 伪造 `data-request` 让父页面请求任意 URL | 父页面只用数字 `userId` 拼 URL，忽略消息中的任何地址 |
| 利用头像地址外联 | 仅允许站内媒体正则，且 CSP `img-src` 限制 |
| 通过链接跳转钓鱼站 | 复用现有外链确认弹窗，展示目标域名 |
| 组件数量爆炸拖慢渲染 | 最多 20 个实例，列表最多 10 条 |
| 越权读取门槛板块内容 | 服务端按查看者 `allowed_forum_ids` 过滤 |

## 7. 规范文档页

### 7.1 入口

- 路由：`/settings/homepage/guide`。
- 装修页顶部操作区新增「组件文档」按钮，跳转该页。
- 装修页新增「插入完整示例」按钮，将示例 HTML 插入编辑框（光标处或替换内容前需确认）。
- 文档页提供「返回装修」入口。

### 7.2 内容结构

1. **快速开始**：自定义主页的最小可用结构。
2. **可用标签**：现有白名单全部标签 + 三个数据组件。
3. **不可用标签**：`script`、`style`、`iframe`、`object`、`embed`、`svg`、`math`、`form`、`input`、`button`、`select`、`textarea`、`video`、`audio`、`canvas`、`meta`、`link`、`base` 等，并说明原因。
4. **属性规则**：
   - 全局属性表。
   - 链接只允许 `http(s)`、`mailto:`、`/` 开头的站内路径、`#` 锚点。
   - 图片只允许 `/forum/api/media/{数字}`。
   - 所有事件属性（`onclick` 等）不可用。
5. **CSS 规则**：列出与 `sanitizer::css_properties()` 一致的属性白名单；说明 `url()`、`@import`、`var()`、`expression()`、CSS 转义、`<style>` 标签不可用。
6. **数据组件**：三个组件的参数、渲染字段、默认值、数量限制与隐私说明（只展示公开资料与公开统计）。
7. **完整示例**：可一键复制的 HTML，覆盖个人资料、统计、帖子、点赞、评论。
8. **常见问题**：样式不生效、图片不显示、组件显示「加载失败」、主页被下架后组件不展示。

文档页内容必须与净化器白名单保持一致；修改任一白名单时同步更新文档页与示例。

### 7.3 完整示例

```html
<div style="max-width: 760px; margin: 0 auto; padding: 24px 16px;">
  <h1 style="margin: 0 0 8px; font-size: 26px;">你好，欢迎来到我的主页</h1>
  <p style="margin: 0 0 24px; color: #666;">这里展示我的公开资料与近期动态。</p>

  <of-profile heading="关于我"></of-profile>
  <of-stats heading="社区数据"></of-stats>

  <of-posts type="authored" limit="5" heading="我的帖子"></of-posts>
  <of-posts type="liked" limit="5" heading="我点赞的帖子"></of-posts>
  <of-posts type="commented" limit="5" heading="我评论过的帖子"></of-posts>
</div>
```

## 8. 权限与边界

| 场景 | 行为 |
| --- | --- |
| 匿名查看公开主页 | 允许，组件按匿名用户可见板块过滤 |
| 登录用户查看 | 允许，按登录用户可见板块过滤 |
| 查看者无权浏览某板块 | 该板块的帖子不出现在任何列表 |
| 帖子被删除 | 不出现在列表，不计入列表条数占位 |
| 评论被删除 | 对应评论不计入 `commented` 列表与统计 |
| 主页被下架 | 非本人/非管理员看到普通主页，组件不渲染；本人与管理员仍看到下架内容与组件 |
| 用户不存在 | `404`，页面正常显示「用户不存在」 |
| 组件数量超限 | 前 20 个正常渲染，其余保持占位文案 |

## 9. 错误处理

| 情况 | 表现 |
| --- | --- |
| 取数请求失败 / 超时 | 所有组件显示「内容加载失败，请刷新重试」 |
| 响应字段缺失 | 跳过该字段或该项，不抛错 |
| 头像地址非法 | 使用占位块，不创建 `<img>` |
| 列表为空 | 显示对应空状态文案 |
| 组件属性非法 | 回退默认值 |
| 引用组件但未登录 | 正常取数；匿名可见内容照常展示 |

## 10. 测试计划

### 10.1 净化器单测

- 三个组件标签与合法属性被保留。
- `type=liked`、`limit=10`、`heading=我的帖子` 保留。
- `type=evil`、`limit=0`、`limit=11`、`limit=abc`、`heading` 超 30 字被移除。
- 组件上的 `onclick`、`onerror`、`href`、`src` 被移除。
- 未知 `<of-foo>` 标签被移除，内部文字保留。
- 现有 8 个用例全部继续通过。

### 10.2 后端接口测试

- 匿名请求成功；未知用户 `404`。
- 响应不含 `email`、`authentications`、认证等级、信息项字段。
- `posts` / `liked` / `commented` 形状与字段正确。
- 门槛板块内容对匿名与无权用户不可见；创建者与管理员可见。
- 删除帖子、删除评论被排除。
- `liked` 仅包含 `value = 1`，同一帖子去重。
- `commented` 按最近评论时间排序并去重。
- 每个列表最多 10 条，且过滤后补足。
- `stats` 数值与构造数据一致。

### 10.3 前端验证

- `vue-tsc --noEmit`、`npm run build` 通过。
- 装修页「组件文档」跳转正确，「插入完整示例」写入完整 HTML。
- 文档页可用标签、不可用标签、属性、CSS、组件参数与示例齐全。
- 浏览器端到端：保存含四类组件的示例 → 个人主页正常渲染资料、统计与三个列表。
- 恶意样例：`<of-posts type="evil" limit="999" onclick="alert(1)">` 渲染为默认配置且无脚本执行。
- 网络面板无外部请求，CSP 与沙箱属性未变化。
- 主页被下架后，非本人访问不渲染组件，也不请求数据接口。
- 既有恶意主页样例仍被拦截，回归通过。

## 11. 上线与迁移

- 无数据库变更，无需数据迁移。
- 后端与前端同批部署，沿用 `deploy-rs.yml`。
- 旧的自定义主页不含组件标签，渲染结果不变。
- 发布后验证：容器重启成功、`homepage-data` 接口可访问、文档页可打开、示例可保存并渲染。

## 12. 验收标准

1. 用户可在自定义主页中使用三个数据组件，展示公开资料、统计、帖子、点赞与评论列表。
2. 规范文档页可从装修页进入，完整说明可用/不可用标签、属性、CSS 与组件参数。
3. 示例代码覆盖四类展示场景，且可一键插入并直接保存使用。
4. 匿名与登录用户都只能看到自己有权浏览的板块内容。
5. 响应不包含邮箱、认证信息与用户信息项。
6. 沙箱、CSP、外链确认、举报下架机制保持原有强度，无新增脚本或外联能力。
7. 非法组件属性、恶意 HTML 与伪造消息均无法造成脚本执行、数据越权或外联。
8. 后端单测、接口测试、前端类型检查与构建全部通过。
