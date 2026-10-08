# Reddit 布局对标重设计 — openforum

- 日期：2026-10-08
- 状态：待评审
- 目标：在现有 openforum（Vue 3 + Element Plus）基础上，把**布局与视觉细节**对标 Reddit 现行 Web 端（Reddit Product Language, RPL），保留 openforum 既有业务（子论坛论主自定义页、认证体系、管理后台）。

## 1. 目标 / 非目标

### 目标
1. 页面骨架与 Reddit 一致：顶栏 + 左侧社区导航栏 + 中间信息流 + 右侧信息卡。
2. 关键组件与 Reddit 一致：帖子卡片（左竖投票列 + 元信息行 + 标题 + 正文预览 + 操作栏）、社区页（封面 + 头像 + Join 按钮 + About 卡）、帖子页（评论区线程线 + 可折叠 + 每层投票列）。
3. 设计令牌对齐 Reddit RPL：OrangeRed 主色、pill 按钮、16px 卡片圆角、卡片描边 + 轻阴影、IBM Plex/系统字体栈。
4. 支持深色模式（跟随系统 + 手动切换）。
5. 响应式：≥1280 三栏 / 960–1280 两栏 / <960 单栏 + 抽屉式左导航。

### 非目标（本轮不做）
- 不做 Reddit 的实时聊天（Chat）、广告位、Mod Tools 全量、Awards/Gold、多语言切换。
- 不重写业务逻辑；不改动认证体系的表单字段与流程（仅跟随新设计令牌换皮）。
- 不引入新 UI 框架（继续用 Element Plus，通过 CSS 令牌覆盖对齐）。

## 2. 现状 vs Reddit（差距清单）

| 区域 | 现状（C·无框紧凑） | Reddit 现行 | 差距 |
|---|---|---|---|
| 页面底色 | 纯白 `#fff`，内容无容器 | 内容区浅灰 `#F6F8F9`，卡片纯白 | 需引入页面底 + 卡片两层次 |
| 卡片 | 无边框无阴影，仅 1px 分隔线 | 16px 圆角 + 1px 描边 + 轻阴影 | 需重做卡片视觉 |
| 顶栏 | 左 logo + 中搜索 + 右按钮，全高 56px | 同结构，但搜索为 pill、右侧有创建/通知/头像 | 细节对齐 |
| 主导航 | 首页左侧仅 3 张侧栏卡（热门/推荐/我的） | 左侧固定社区导航栏：Home/Popular/All + 社区列表 + Create | 需新建左导航栏 |
| 右侧栏 | 首页无 | 社区页/帖子页有 About 卡、相关社区 | 需新建右栏 |
| 信息流排序 | 文字 tab（热门/最新/最高分） | pill chips（Best/Hot/New/Top/Rising） | 改 pill |
| 帖子卡片 | 左投票列 ✔ | 左投票列 + 底部操作栏（评论/分享/保存/更多） | 补操作栏 |
| 社区页 | 3px 色条 + 56px 方图标 + Join | 大封面 + 叠加圆形头像 + Join pill + 统计行 | 重做头部 |
| 帖子页 | 单列，无评论排序 | 帖子卡 + 评论排序 chips + 线程线 + 折叠 | 补排序/线程线/折叠 |
| 投票图标 | 文字 ▲ ▼ | SVG 箭头，hover 有胶囊底色 | 换 SVG |
| 深色模式 | 无 | 完整深色 | 新增 |

## 3. 设计令牌（对齐 Reddit RPL live-extract）

覆盖 `main.css` 的 `:root`，并新增深色覆盖。保留 `--of-` 前缀以免大面积改名（值换成 Reddit 值），同时新增语义别名。

```css
:root {
  /* 品牌与交互 */
  --of-primary: #d93900;          /* CTA / 赞同 active（Reddit 交互橙） */
  --of-primary-hover: #ae2c00;
  --of-brand: #ff4500;            /* logo / 强调 */
  --of-upvote: #d93900;
  --of-downvote: #6a5cff;
  --of-downvote-hover: #523dff;
  --of-link: #115bca;
  --of-link-hover: #0a449b;

  /* 中性层次 */
  --of-bg: #f6f8f9;               /* 页面底 */
  --of-card: #ffffff;             /* 卡片/容器 */
  --of-surface: #e5ebee;          /* 次级填充（按钮 hover、flair 底） */
  --of-surface-weak: #eef1f3;
  --of-border: #e5ebee;           /* 分隔/描边 */
  --of-text: #181c1f;             /* 前景 strong */
  --of-text-body: #333d42;
  --of-text-muted: #5c6c74;       /* 次要/元信息 */
  --of-text-subtle: #8b979e;

  /* 形状 */
  --of-radius: 16px;              /* 卡片 */
  --of-radius-md: 8px;
  --of-radius-sm: 4px;
  --of-radius-pill: 9999px;       /* 按钮/搜索/flair */

  /* 阴影（RPL） */
  --of-shadow-card: 0 1px 4px rgba(0,0,0,.149), 0 4px 4px rgba(0,0,0,.06);
  --of-shadow-nav: 0 4px 14px rgba(0,0,0,.1);
  --of-shadow-modal: 0 2px 15px rgba(26,26,27,.3);

  /* 排版 */
  --of-font: -apple-system, system-ui, 'Segoe UI', Roboto, 'PingFang SC',
             'Hiragino Sans GB', 'Microsoft YaHei', Helvetica, Arial, sans-serif;
  --of-fs-title: 18px;  --of-fw-title: 600;
  --of-fs-body: 14px;   --of-fs-meta: 12px;

  /* 尺寸 */
  --of-nav-height: 56px;
  --of-rail-left: 270px;
  --of-rail-right: 316px;
  --of-feed-width: 756px;
}

:root.dark {
  --of-bg: #0b1416;
  --of-card: #181c1f;
  --of-surface: #223237;
  --of-surface-weak: #1a282d;
  --of-border: #2a3c42;
  --of-text: #eef1f3;
  --of-text-body: #d7dde0;
  --of-text-muted: #96a6ac;
  --of-text-subtle: #7c8c92;
  --of-link: #7bb0ff;
  --of-shadow-card: none;
}
```

Element Plus 令牌同步：`--el-color-primary: #d93900`、`--el-border-radius-base: 8px`、`--el-border-radius-round: 9999px` 等。

**旧令牌别名（必须保留，避免逐个改调用点）**：现有 CSS 与 `.vue` 里大量引用 `--of-primary-soft`、`--of-border-soft`、`--of-radius-sm`、`--of-shadow`、`--of-primary-dark`、`--of-card`。新 `:root` 中这些名字必须继续存在并指向新语义：

```css
--of-primary-soft: var(--of-surface);   /* 原 #f2f2f2 → 次级填充 */
--of-border-soft: var(--of-border);     /* 原更轻分隔 → 现统一描边 */
--of-primary-dark: var(--of-primary-hover);
--of-shadow: var(--of-shadow-card);
```

## 4. 页面布局规格

### 4.1 页面骨架（App.vue）
```
<AppShell>
  <TopNav/>                       /* sticky, 56px, 白底/深色底, 底部 1px 描边 */
  <main class="app-main">         /* 三栏 grid */
    <router-view :rail-left :rail-right/>  /* 由各页面决定是否渲染左右栏 */
  </main>
</AppShell>
```
栅格：
- `≥1280px`：`grid-template-columns: var(--of-rail-left) minmax(0, 1fr) var(--of-rail-right)`，gap 24px，整体 max-width 1600px 居中。
- `960–1280px`：隐藏右栏 → `270px minmax(0,1fr)`。
- `<960px`：单列，左栏改为抽屉（顶栏汉堡按钮触发 `el-drawer`）。

### 4.2 顶栏 TopNav
- 左：汉堡（<960 显示）+ logo（Snoo 风格标记，`--of-brand` 圆角方块 + 「OpenForum」词标）+ 首页图标链。
- 中：搜索 pill（`--of-surface` 底、无描边、focus 时 1px `--of-link` 描边、宽 flex 1 max 690px），回车跳 `/search`。
- 右：`创建` pill 主按钮（`--of-primary` 底白字）+ 通知（图标，暂用 `el-badge` 占位）+ 用户头像下拉；未登录时显示「登录」pill 主按钮 + 「注册」pill 次按钮。
- 高度 56px，`position: sticky; top: 0`，`z-index: 100`。

### 4.3 首页信息流 `/`
- 左栏（`LeftRail`，新组件，替代现有 `ForumSidebar` 的首页用法）：
  1. 导航区：首页 / 热门 / 全部（图标 + 文字行，当前项高亮 `--of-brand` 图标）
  2. `我的社区`：登录后列出 `isMember` 的社区（圆形 32px 图标 + `f/xxx`），空态「还没有加入社区」
  3. `热门社区`：取 `forumApi.popular(8)`
  4. `为你推荐`：取 `forumApi.recommended(8)`
  5. 底部 `+ 创建社区` pill 按钮 + 资源链接（关于 / 帮助）
- 中间信息流：
  - 顶部排序 chips（热门 / 最新 / 最高分）+ 右侧「刷新」图标按钮
  - 帖子卡片列表（见 §5.1）
  - 底部「加载更多」
- 右栏（首页可选，`≥1280` 显示）：`今日热门` 社区榜 + 站点介绍卡。默认渲染，与本轮 F-01 对齐。

### 4.4 社区页 `/f/:slug`
- 页面容器为圆角卡片（`--of-radius`）：
  - **封面**：高 128px（移动 80px），底色取 `forum.themeColor`，无图时用主题色渐变。
  - **头部行**：圆形头像 80px（叠加封面下沿 `-16px`）+ 社区名 `f/name`（`--of-fs-title` 加大）+ `加入/已加入` pill 按钮（加入=主色实心，已加入=描边）。
  - **统计行**：`{memberCount} 成员 · {postCount} 帖子 · {createdAt 创建}`，`--of-text-muted`。
  - **门槛提示**：`requiredAuthTypes` 非空时，展示 `el-alert`（保留现有逻辑）。
- 主体两栏：左（`minmax(0,1fr)`）帖子流；右栏（316px）**About 卡**：简介、创建时间、成员/帖子数、`+ 创建帖子` pill 按钮、规则（`forum.rules`）。
- 自定义页面（`customPageStatus === 'APPROVED'`）时：整页渲染 `CustomPageRenderer`，保留现状，不套 About 卡。
- 默认模板的 `template-section`（公告/分区/链接）放入 About 卡下方的「社区信息」卡内。

### 4.5 帖子详情页 `/post/:id`
- 主卡：与信息流帖子卡同款（含左投票列 + 操作栏），标题用 `--of-fs-title` 放大到 24px。
- 评论排序 chips：`最佳 / 最新 / 最高分`（**前端排序**：对后端返回的树按需要重排，见 §6）。
- 发表评论框：卡片化，`RichTextEditor` 保留。
- 评论树：见 §5.3。
- 右栏（`≥1280`）：About 社区卡（同 4.4）+ 相关帖子。

### 4.6 其它页
- `/search`：顶部搜索框（复用顶栏样式）+ 结果 tabs（帖子/社区）改为 pill 分段控件；结果用帖子卡片/社区行。
- `/u/:id`：用户头部卡（封面 + 圆形头像 + 用户名 + `el-tag` 认证徽章行 + 简介），下方 tabs：`帖子 / 评论 / 关于`。**帖子/评论 tab 依赖新增后端端点**（见 §6），无数据时给空态。
- `/login`、`/register`：保持现有 `auth-*` 结构，仅套用新令牌（卡片化 + pill 按钮 + 主色）。
- `/admin`、`/authentication*`、`/create-forum`、`/f/:slug/settings`：只做令牌与卡片化换皮，不改结构。

## 5. 组件规格

### 5.1 PostCard
```
article.post-card        /* 白底 16px 圆角 1px 描边 轻阴影，hover 描边加深 */
  .post-vote             /* 40px 竖列：SVG 上箭头 / 分数 / SVG 下箭头，各自 hover 胶囊底 */
  .post-main
    .post-meta           /* f/社区（主色链接） · u/作者 · 时间 · 置顶/flair 徽章 */
    h3.post-title        /* 18px/600，hover 主色 */
    .post-preview        /* 2 行截断正文预览 */
    .post-actions        /* 评论数 / 分享 / ⋯（hover 胶囊底），与正文用 1px 分隔线隔开 */
```
- 投票箭头改为内联 SVG（Reddit 造型：圆角三角 + 短柄），点击区域 32×32，hover 底 `--of-surface`。
- 已赞同：箭头与分数均 `--of-upvote`；已反对：`--of-downvote`。
- 分享：`navigator.share` 可用则调用，否则复制链接并 `ElMessage` 提示（纯前端）。

### 5.2 VoteColumn（信息流/帖子页左侧）
- 竖排 40px，间距 2px；`disabled` 时整体 50% 透明 + `not-allowed`。
- 分数 `tabular-nums`，14px/600。

### 5.3 CommentNode
```
.comment
  .comment-vote          /* 竖列：SVG 箭头 + 分数，同 VoteColumn 缩小版 */
  .comment-main
    .comment-head        /* u/作者（主色链接） · 时间 · 折叠按钮（▾/▸） */
    .comment-body
    .comment-actions     /* 回复 / 分享 / 举报（占位） */
  .comment-children      /* 递归；左侧 2px 线程线，hover 线程线高亮 */
```
- **线程线**：`.comment-children` 用 `border-left: 2px solid var(--of-border)`，hover 时变 `--of-text-muted`（Reddit 的「thread line」）。
- **折叠**：头部左侧圆形 `−/+` 按钮，折叠后仅显示一行「{作者} · {分数} · {n} 条回复」，点击展开。
- 回复层级：沿用现有 `depth < 2` 限制；更深层「查看更多回复」跳转或展开（本轮沿用 depth<2）。

### 5.4 SortChips（新）
- 通用 pill 分段：`最佳 / 热门 / 最新 / 最高分`（按页面裁剪），active 为主色文字 + `--of-surface` 底。

### 5.5 图标
- 新增 `components/icons/` 目录，纯内联 SVG 组件：`UpArrow`、`DownArrow`、`Comment`、`Share`、`More`、`Search`、`Home`、`Bell`、`Plus`、`Menu`、`Collapse`。统一 20×20，`currentColor`。

## 6. 后端配套（最小必要）

优先级：**布局本身全部为前端改动**；以下为提升 Reddit 一致性所需的增量。

| 编号 | 需求 | 现状 | 方案 | 必需性 |
|---|---|---|---|---|
| B-01 | 用户主页「帖子/评论」列表 | Repository 已有 `findByAuthorIdAndRemovedFalse`，无端点 | 新增 `GET /users/{id}/posts`、`GET /users/{id}/comments`（分页，返回 PostSummaryDTO / CommentDTO 扁平列表） | 中 |
| B-02 | 评论排序 | 后端只按 createdAt 升序返回整树 | 前端排序（最佳/最新/最高分对同级子节点重排），不改后端 | 低（前端实现） |
| B-03 | 帖子搜索 | 无 | 新增 `GET /posts?q=`（`title LIKE`），前端 `/search` 已能承载 | 低 |

本轮默认只做 B-02（前端）；B-01/B-03 作为可选后续，若用户要求则一并实现。

## 7. 响应式与深色

- 断点：`1280px`（收起右栏）、`960px`（收起左栏 → 抽屉）、`640px`（单列、顶栏两行、字号降级）。
- 深色：`:root.dark` 覆盖令牌；`stores/ui.ts`（新增）持久化到 `localStorage['of-theme']`，首屏读取 `prefers-color-scheme`；顶栏用户菜单加「深色模式」开关。
- Element Plus 深色：引入 `element-plus/theme-chalk/dark/css-vars.css` 并给 `html` 加 `dark` class。

## 8. 实施顺序

1. **S1 令牌与全局**：`main.css` 令牌替换 + 深色 + 卡片/按钮基线；`stores/ui.ts`；Element Plus 深色接入。
2. **S2 骨架**：`App.vue` 三栏栅格；`TopNav` 重做；图标组件。
3. **S3 首页**：`LeftRail` 新组件 + 信息流 + 右栏。
4. **S4 帖子卡与投票**：`PostCard`、`VoteColumn`、`SortChips`。
5. **S5 社区页**：封面/头像/Join/About 卡/自定义页兼容。
6. **S6 帖子页与评论**：评论排序 + `CommentNode` 线程线/折叠。
7. **S7 其它页换皮**：搜索 / 用户主页 / 登录注册 / 认证 / 管理后台 / 建论坛 / 设置。
8. **S8 验证**：`type-check` + `build` + 11 路由移动端体检 + 深色截图比对 + 部署。

## 9. 验收标准

- 视觉：首页 / 社区页 / 帖子页三处与 Reddit 截图逐区块可比（顶栏、左栏、卡片、投票列、操作栏、About 卡、评论线程线）。
- 交互：投票、排序、加入/退出、折叠评论、分享、深色切换、移动端抽屉均可用。
- 工程：`npm run type-check` 与 `npm run build` 通过；无横向溢出（390×844 实测 overflow=0）。
- 部署：GitHub Actions 流水线成功，线上资源指纹更新。

## 10. 风险

- Element Plus 与 RPL 的圆角/主色差异需要令牌覆盖 + 少量 `!important`，注意深色下对比度。
- `main.css` 从「无框」改「卡片」是全局视觉变更，需逐页回归（认证/管理后台共 6 个页面）。
- 评论折叠与排序为纯前端实现，不改后端；如后续要求「加载更多回复」，需后端分页支持。
