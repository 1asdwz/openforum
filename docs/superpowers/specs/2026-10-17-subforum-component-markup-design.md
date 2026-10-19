# 子论坛「受限组件标记语言」设计稿

- 日期：2026-10-17
- 状态：草案，待评审
- 关联：`2026-10-17-subforum-custom-layout-design.md`
- 路线：**自由代码写法 + 白名单组件词汇表**（不是任意 HTML，也不是纯表单 Schema）
- 目标：坛主可用「代码」自由编排子论坛板块，但**只能使用平台集成好的组件**，从根上防止恶意欺诈 / 诱骗代码

---

## 1. 设计思想

> 给坛主一个**受控的词汇表**，而不是一门外语的全部单词。

- **能自由编排**：像写 HTML 一样组合组件、嵌套、排序 → 保留"写代码"的表达力与手感。
- **不能创造危险件**：标签只能是平台注册的 `of-*` 组件；没有 `<script>/<form>/<iframe>` 等任意标签，也没有任意 CSS/JS。
- **越界的不是"被静默删掉"，而是"直接被拒绝"**：提交时给出精确报错，反向教会坛主写对。

与前一版「自由 HTML+CSS」的关键区别：

| | 自由 HTML+CSS | 受限组件标记（本方案） |
|---|---|---|
| 标签 | 白名单清洗，漏网靠沙箱兜底 | **只允许 `of-*` 组件，其余直接拒绝** |
| 样式 | 任意 CSS（需 CSS 净化对抗） | **不开放 CSS**，样式由组件 props + 主题令牌控制 |
| 欺诈面 | 可伪造登录框、全屏遮罩 | **没有表单/输入类组件，物理上造不出钓鱼 UI** |
| 渲染 | iframe 沙箱（牺牲 SEO/性能） | **主文档内渲染（AST→可信组件），SEO/性能/a11y 皆优** |
| 审核 | 必须人工 | **可自动放行**（攻击面已被词汇表封死） |

---

## 2. 总体架构

```
坛主编辑器（代码框）
   │  写受限 markup：<of-section><of-hero .../><of-post-list .../></of-section>
   ▼
[服务端] MarkupValidator ── 依赖 ──> ComponentRegistry（组件目录 = 唯一真值）
   │   逐标签/逐属性校验，非法即 400 + 精确报错
   │   通过 → 生成 AST (JSON)
   ▼
存储：raw_markup + page_ast（AST 为渲染真值）
   ▼
[前端] ComponentRenderer ── resolveComponent(tag) ──> 平台可信 Vue 组件
   │   递归渲染 AST，绝不 v-html，绝不执行用户内容
   ▼
子论坛页面（主文档 DOM，无 iframe）
```

**核心原则**：用户输入 = 数据；平台组件 = 代码。二者永不混淆。

---

## 3. 组件目录（Component Registry）

> 组件目录是**唯一真值**，同时驱动：服务端校验、前端渲染、编辑器自动补全。新增组件只需注册一次。

### 3.1 组件定义格式
```
key          of-hero                      唯一标签名
label        头图区块                     编辑器显示名
category     layout | content | forum | media | link
accepts      none | text | blocks         子内容策略
props        [ {name, type, required, default, doc} ]
renderer     前端组件名 / 服务端渲染器
```

### 3.2 Prop 类型系统（全部经服务端校验）
| 类型 | 说明 | 校验 |
|---|---|---|
| `text` | 纯文本 | 长度上限 + HTML 转义 + 内容审核 |
| `richtext` | 有限内联富文本 | 仅 `strong/em/a/br`，转义后转 AST |
| `url` | 链接 | 协议白名单 + 域名策略（§5.3） |
| `image` | 图片地址 | 协议 + 尺寸提示，禁 `data:text/html` |
| `enum` | 枚举 | 必须命中允许值 |
| `int` | 整数 | min/max |
| `bool` | 布尔 | `true/false` |
| `token` | 样式令牌引用 | 仅允许注册过的令牌名（§4） |

### 3.3 首批内置组件（示例，非穷举）
| 标签 | accepts | 关键 props | 用途 |
|---|---|---|---|
| `of-section` | blocks | `padding=sm/md/lg` | 分区块 |
| `of-columns` | blocks | `ratio=1-1/2-1/1-2` | 多栏布局 |
| `of-column` | blocks | — | 栏容器 |
| `of-hero` | none | `title,subtitle,image,align` | 头图 |
| `of-heading` | text | `level=1/2/3` | 标题 |
| `of-text` | richtext | `align` | 正文 |
| `of-image` | none | `src,alt,caption` | 图片 |
| `of-announcement` | text | `title` | 公告条 |
| `of-rules` | none | `title` | 读取板块规则 |
| `of-post-list` | none | `limit(1-50),sort=hot/new/top` | 帖子流 |
| `of-members` | none | `limit` | 成员数/列表 |
| `of-join-button` | none | — | 加入按钮（平台行为） |
| `of-gallery` | none | `images[],cols` | 图集 |
| `of-link` | text | `href` | 安全外链（强制标 + rel） |
| `of-link-list` | blocks(`of-link`) | — | 链接组 |
| `of-divider` / `of-spacer` | none | `size` | 分隔 |

**明确不提供**：表单、输入框、密码框、按钮跳转外站、全屏遮罩、自定义脚本、任意 iframe/embed。

---

## 4. 样式：令牌取代 CSS

不开放 CSS，改为**设计令牌（Design Tokens）**：

- 平台注册一组令牌：`color.primary`、`color.surface`、`radius.md`、`space.lg`、`font.heading` …
- 组件 props 只能引用令牌名（`token` 类型），落地由平台完成。
- 坛主可**覆盖令牌取值**（选色/圆角），仍受平台约束（如颜色空间、对比度检查）。
- 结果：**深浅色、响应式、无障碍由平台统一保证**，坛主仍能做出差异化的"样式设计"。

```
<!-- 允许：引用令牌 -->
<of-section padding="lg" tone="surface">
<!-- 拒绝：内联样式 / class / style 属性 -->
<of-section style="position:fixed;top:0">   ← 400 报错
```

---

## 5. 校验与防欺诈

### 5.1 服务端校验流水线（权威）
1. **解析** markup → DOM 树（容错解析器）。
2. **标签校验**：非 `of-*` 或未注册 → **拒绝并报错**（不静默删除）。
3. **嵌套校验**：按 `accepts` 规则（`blocks` 才能有子块；`of-link-list` 只收 `of-link`）。
4. **属性校验**：未知属性拒绝；已知属性按类型校验（enum 命中、int 范围、url 协议…）。
5. **文本提取**：所有 `text/richtext` 进入内容审核队列（AI `ReviewPlugin`）。
6. **限额**：总大小 ≤ 64 KB、节点数 ≤ 500、嵌套深度 ≤ 8、同类组件数量上限。
7. **产物**：`raw_markup` + 规范化 `AST`（AST 为渲染唯一来源）。

### 5.2 报错 UX（提交即教学）
```json
{
  "code": 400,
  "message": "标记校验未通过",
  "data": {
    "errors": [
      { "line": 3, "tag": "div",     "message": "未知标签 <div>，可用：of-section, of-hero, …" },
      { "line": 7, "tag": "of-post-list", "message": "属性 sort=\"popular\" 非法，可选：hot|new|top" },
      { "line": 9, "tag": "of-image", "message": "src 协议不允许：javascript:" }
    ]
  }
}
```

### 5.3 反欺诈专项
| 风险 | 本方案如何封死 |
|---|---|
| 伪造登录/支付表单 | **目录里没有任何 form/input 组件**，造不出 |
| 钓鱼外链 | 仅 `of-link`，强制 `rel="noopener nofollow"` + 显式外链图标 |
| 冒充平台 UI | 组件统一渲染 + 内容区带 **"由坛主提供"边界标识**，与平台 chrome 视觉区隔 |
| 全屏遮罩 / 点击劫持 | 不提供 `position:fixed` 类组件，布局组件受容器约束 |
| 诱骗文案 | 文本节点过 AI 审核（密码/验证码/官方等敏感诱导词） |
| 资源耗尽 | 大小/节点/深度硬上限 |
| 越权改页 | 复用 `can_manage`（坛主/管理员） |
| 数据外泄 | 图片/外链走域名白名单或本站代理 |

> 因为攻击面被词汇表**物理封死**，这套方案**可自动放行**（或仅首帖轻审），大幅降低运营审核成本。

---

## 6. 数据模型

### 6.1 `sub_forums`（扩展）
```
+ page_mode      VARCHAR(16)   -- TEMPLATE(旧) | COMPONENT(新)
+ page_markup    LONGTEXT      -- 坛主原始受限 markup
+ page_ast       JSON          -- 校验后 AST（渲染真值）
+ page_status    VARCHAR(16)   -- DRAFT/PENDING/APPROVED/REJECTED（沿用）
+ page_version   INT
```
- 旧字段 `template_config`（L1）与 `custom_page_html`（L4）**保留兼容**：`page_mode` 缺省回退旧逻辑。

### 6.2 `forum_page_versions`（版本/回滚/diff，沿用前稿）
```
id, forum_id, version, markup, ast, status, review_note, created_by, created_at
UNIQUE(forum_id, version)
```

---

## 7. API 契约

| 方法 | 路径 | 权限 | 说明 |
|---|---|---|---|
| GET | `/forums/component-catalog` | 公开 | 返回组件目录 JSON（编辑器补全/文档共用真值） |
| GET | `/forums/{slug}` | 公开 | 详情，含 `pageAst`（仅 APPROVED 时下发） |
| PUT | `/forums/{id}/page` | 坛主 | body `{markup}` → 校验 → 存 DRAFT，返回 `{ast, errors[]}` |
| POST | `/forums/{id}/page/submit` | 坛主 | 提交审核（或自动放行） |
| GET | `/forums/{id}/page` | 坛主/管理员 | 返回 `{markup, ast, status, version}` |
| GET | `/forums/{id}/page/versions` | 坛主/管理员 | 版本列表 |
| POST | `/forums/{id}/page/restore` | 坛主 | body `{version}` → 旧版为新的 DRAFT |
| GET/POST | `/admin/custom-pages*` | 管理员 | 审核台（沿用） |

响应沿用 `{code, message, data}`。

---

## 8. 前端渲染与编辑器

### 8.1 渲染器 `ComponentRenderer.vue`
```ts
const registry = new Map<string, Component>([
  ['of-section', SectionBlock], ['of-hero', HeroBlock], /* … */
])
function render(node: AstNode) {
  const C = registry.get(node.tag) ?? UnknownBlock   // 未知走占位，不注入
  return h(C, node.props, renderChildren(node.children))
}
```
- **全程 `h()` 渲染，绝不 `v-html`** → 不存在 HTML 字符串注入。
- 未知标签降级为占位提示，不影响整页。
- 主文档渲染 → SEO、a11y、性能、深色模式全继承平台。

### 8.2 编辑器 `ComponentEditor.vue`
- CodeMirror 6：`of-*` 标签 + 属性**自动补全**（数据来自 `component-catalog`），HTML 语法高亮。
- **实时校验**：本地调 `/forums/{id}/page` 的 dry-run 或内联规则，错误波浪线 + 行号。
- **实时预览**：右侧用同一 `ComponentRenderer` 渲染 AST。
- 侧栏**组件面板**：点选插入组件（零代码坛主也能用）+ 文档。
- 操作：保存草稿 / 提交 / 版本历史 / 回滚。

---

## 9. 与旧方案对比（决策依据）

| 维度 | 自由 HTML+CSS（前稿） | 受限组件标记（本稿） |
|---|---|---|
| 安全 | 需 sanitizer 长期对抗 + 沙箱 | 词汇表封死，攻击面可枚举 |
| 审核 | 必须人工+AI | **可自动放行** |
| SEO/性能 | iframe 差 | 主文档优 |
| 样式自由 | 高（但风险高） | 中（令牌级别，够用且安全） |
| 表达力 | 无限 HTML/CSS | 组件组合级（覆盖 90% 需求） |
| 开发成本 | 低前期/高长期 | 中前期（组件库）/低长期 |

> 结论：**采用本方案**。它把"自由代码"的表达力保留在"编排层"，把"危险"锁死在"组件层"。

---

## 10. 实施计划

**Phase 0（骨架）**
1. `ComponentRegistry` + `MarkupValidator`（服务端，唯一真值）。
2. `GET /forums/component-catalog`。
3. 首批 6 个组件：`of-section / of-heading / of-text / of-image / of-link / of-post-list`。
4. `page_mode=COMPONENT` 存储 + 校验接口。

**Phase 1（体验）**
5. `ComponentRenderer.vue`（AST→组件，主文档渲染）。
6. CodeMirror 编辑器 + 自动补全 + 实时预览 + 组件面板。
7. `forum_page_versions` 版本/回滚/diff。

**Phase 2（扩展）**
8. 组件扩到 ~20 个（`of-hero/of-columns/of-gallery/of-rules/of-join-button…`）。
9. 令牌主题覆盖（选色/圆角/明暗）。
10. 文本 AI 审核接入 + 自动放行策略。

---

## 11. 验收 / 测试用例

| 用例 | 期望 |
|---|---|
| 提交含 `<div>` / `<script>` / `<form>` | 400，报错指明未知标签 |
| `<of-post-list sort="popular">` | 400，枚举非法 |
| `<of-image src="javascript:...">` | 400，协议非法 |
| `<of-section style="position:fixed">` | 400，不支持 style |
| `<of-link href="https://x.com">` | 通过，渲染带 `rel=noopener nofollow` + 外链标识 |
| 5000 个节点 | 400 超限 |
| 非坛主调用 | 403 |
| 未过审访问子论坛 | 回退旧模板 |
| 正常组合组件 | 主文档内正确渲染，深色模式/移动端自适应 |
| 未知但格式合法的标签 | 前端降级占位，不崩溃、不注入 |

---

## 12. 非目标
- 任意 HTML / 任意 CSS / 任意 JS（明确禁止）。
- 坛主自定义组件或上传组件代码。
- 独立域名托管子论坛页。
