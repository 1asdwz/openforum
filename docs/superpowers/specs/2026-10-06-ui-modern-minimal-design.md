# 界面现代简约化（方向 C · 无框紧凑）— 设计规格

- 日期：2026-10-06
- 状态：已与用户确认，待实现
- 范围：前端（Vue 3 + Element Plus），以 `src/styles/main.css` 为主战场，全站统一
- 前置：性能优化（首屏骨架屏 + Element Plus 按需引入，已上线）

## 1. 背景与目标

当前 UI 为 Reddit 风格：浅灰底 `#f6f7f8` + 橙色主色 `#FF4500` + 卡片盒子（边框 + 圆角 10px + 阴影）。
用户要求「优化界面，现代简约」，在三个可视化方向中选定 **方向 C · 无框紧凑**：

> 彻底去掉卡片盒子，用分隔线组织信息，密度最高、最冷静克制的排版派极简。

目标是把"卡片堆叠"改为"发丝线分隔的排版流"，强调信息层级与留白节奏，而非盒子与阴影。

## 2. 关键决策（已与用户确认）

| 决策点 | 结论 |
| --- | --- |
| 设计方向 | C · 无框紧凑（3 个候选方向中选定） |
| 覆盖范围 | 核心浏览链路：首页、子论坛页、帖子详情页 |
| 影响面 | **全站一并统一**（`main.css` 为全局样式出口，其他页面同步变更，避免两套视觉并存） |
| 强调色 | `#0a0a0a` 黑（原橙色退居为"仅投票用"的暖色） |
| 投票色 | **保留双色**：赞同暖橙、反对蓝（C 风格中唯一的彩色信号） |
| 子论坛区布局 | 无框行列表（两列，行间发丝线），取代原卡片网格 |
| 浮层处理 | 下拉菜单/气泡保留白底 + 轻阴影 + 8px 圆角（浮层需要从页面"浮起来"） |

## 3. 设计语言（Design Tokens）

### 3.1 颜色

| 变量 | 现值 | 新值 | 说明 |
| --- | --- | --- | --- |
| `--of-bg` | `#f6f7f8` | `#ffffff` | 纯白画布 |
| `--of-card` | `#ffffff` | `#ffffff` | 与底色同色 → 卡片不再靠底色区分 |
| `--of-border` | `#e5e6e8` | `#e8e8e8` | 发丝分隔线 |
| `--of-border-soft` | `#edeff1` | `#f0f0f0` | 更轻的分隔/悬停底 |
| `--of-text` | `#1a1a1b` | `#0a0a0a` | 主文本 |
| `--of-text-muted` | `#7c7c7c` | `#6b7280` | 次级文本 |
| `--of-text-subtle` | `#a8a8a8` | `#9ca3af` | 三级文本 |
| `--of-primary` | `#ff4500` | `#0a0a0a` | 强调色 = 黑 |
| `--of-primary-dark` | `#d93a00` | `#000000` | 强调色悬停 |
| `--of-primary-soft` | `#fff0eb` | `#f2f2f2` | 激活态底色（灰） |
| `--of-upvote` | `#ff4500` | `#e8590c` | 赞同（保留暖色，降饱和以适配黑白基调） |
| `--of-downvote` | `#7193ff` | `#5b7cfa` | 反对 |
| `--of-radius` | `10px` | `4px` | 该变量仅供控件（按钮/输入框/标签）使用 |
| `--of-radius-sm` | `6px` | `4px` | 同上 |
| `--of-shadow` | `0 1px 3px rgba(0,0,0,.06)` | `none` | 卡片无阴影 |

> 说明：所有"卡片类"规则（`.post-card`、`.forum-card`、`.sidebar-card`、`.content-card`、`.post-detail-card`、`.comment-*` 等）将显式写 `border: none; border-radius: 0; box-shadow: none`，不再依赖 `--of-radius`。
>
> 注意：`--of-primary` 同时被子论坛图标用作默认底色（`f.themeColor || 'var(--of-primary)'`）。变黑后默认图标为黑底白字，与 C 语言一致；用户自建子论坛若设置了 `themeColor` 则不受影响。

### 3.2 字体层级

| 用途 | 规格 |
| --- | --- |
| 页面主标题（h1） | `22px / 700 / letter-spacing -0.02em` |
| 帖子标题（列表） | `17.5px / 700 / -0.01em` |
| 帖子标题（详情） | `24px / 700 / -0.02em` |
| 区块小标题 | `12px / 700 / uppercase / letter-spacing 0.09em / muted` |
| 正文 | `14px / line-height 1.65` |
| 元信息（meta） | `12px / muted` |
| 数字（分数、计数） | `font-variant-numeric: tabular-nums` |

### 3.3 间距与节奏

| 项 | 规格 |
| --- | --- |
| 页面容器 | `max-width 1180px`，`padding 28px 24px 60px` |
| 主布局 | `grid: 240px minmax(0,1fr)`，`gap 28px` |
| 侧栏 | `gap 26px`（分组间用发丝线 + 28px 间距） |
| 帖子流 | `gap 0`，单条 `padding 16px 0` + 底部发丝线 |
| 区块下间距 | `26px` |
| 顶栏高度 | `56px`（不变） |

### 3.4 动效

仅保留 `0.15s` 的 `background / color / border-color` 过渡。**移除** `transform: translateY(-1px)` 之类的卡片位移与 `box-shadow` 悬停放大（与无框语言不符）。

## 4. 逐页结构改造

### 4.1 首页（`Home.vue` + `ForumSidebar.vue` + `PostCard.vue`）

| 元素 | 现状 | C 方案 |
| --- | --- | --- |
| `.forum-grid` | `repeat(auto-fill, minmax(260px,1fr))` 卡片网格 | 两列（`1fr 1fr`），窄屏单列 |
| `.forum-card` | 白卡 + 边框 + 圆角 + hover 位移 | 无框行：`padding 14px 0`，底部发丝线，无 hover 位移（仅图标底色微变） |
| `.feed-toolbar` | 白卡片条 | 无框工具行：`padding 0 0 12px`，底部发丝线，`margin-bottom 4px` |
| `.sort-tabs button.active` | 橙底浅橙 | 灰底 `#f2f2f2` + 黑字 |
| `.post-card` | 白卡 + 边框 + 圆角 + hover 阴影 | 无框行：`padding 2px 0`，底部发丝线，无阴影 |
| `.vote-col` | 44px 灰底 + 右边框 | 40px，透明底、无右边框 |
| `.sidebar-card` | 白卡 + 边框 + 圆角 | 无框分组：组标题为小号大写标签，分组间发丝线 |
| 区块标题 | `16px` 普通 | 小号大写标签（见 3.2） |

### 4.2 子论坛页（`ForumPage.vue`）

| 元素 | 现状 | C 方案 |
| --- | --- | --- |
| `.forum-hero` | 白卡 + 边框 + 84px 彩色横幅 | 去边框；横幅压扁为 **3px 顶部强调条**（取 `forum.themeColor`） |
| `.forum-hero-icon` | 72px + 4px 白边 + 圆形 | 56px + 8px 圆角 + 无白边 |
| `.forum-hero-body` | `padding 0 20px 18px` | `padding 18px 0 14px` |
| `.forum-hero-info h1` | 24px | 24px / 700 / -0.02em |
| `.template-section` | 白卡 + 边框 | 无框分组：小号大写标题 + 发丝线分隔 |
| `.section-title` | `16px` | 小号大写标签 |

### 4.3 帖子详情页（`PostDetail.vue` + `CommentNode.vue`）

| 元素 | 现状 | C 方案 |
| --- | --- | --- |
| `.post-detail-card` | 白卡 + 边框 + 圆角 | 去框；标题与正文按 3.2 排版 |
| `.post-detail-title` | 22px | 24px / 700 |
| `.comment-box` | 白卡 + 边框 | 去框 |
| `.comment-node` | 白卡 + 浅边框 | 去框，行间发丝线 |
| `.comment-replies` | `2px solid var(--of-border-soft)` 左竖线 | 保留，改为 `2px solid var(--of-border)` 提高对比 |
| `.post-detail-content` | 14–15px | `15px / line-height 1.7` |

### 4.4 全站其他页面

以下页面共用同一批 class，随 `main.css` 一并切换为 C 风格，**不改模板**：
登录、注册、个人资料、认证中心、认证等级向导、子论坛设置、创建子论坛、搜索、管理后台。

涉及的无框化改写：`.content-card`、`.auth-card`、`.record-item`、`.state-box`、`.admin-preview-frame`、`.custom-page-frame` 等。

> `state-box`（空态/错误态）保留虚线的"占位感"，但改为更浅的灰；错误态保留红色语义。

## 5. Element Plus 组件对齐

在 `main.css` 的 `:root` 覆盖 EP 变量，并补充少量选择器微调：

| 组件 | 处理 |
| --- | --- |
| 全局 | `--el-color-primary: #0a0a0a` 及其 `light-3/5/7/8/9`、`dark-2` 阶梯；`--el-border-radius-base: 4px` |
| `el-button` | 高度 34px；primary = 黑底白字；default = 透明底 + 发丝线 + 黑字；圆角 4px |
| `el-input` / `el-textarea` | 浅灰底 `#f4f4f4`、无边框、聚焦时 1px 黑描边 |
| `el-tag` | 去彩色：灰底黑字（`effect="plain"` 保持发丝线样式） |
| `el-dropdown-menu` / `el-popper` / `el-tooltip` | **保留浮层语义**：白底、8px 圆角、轻阴影、发丝线 |
| `el-skeleton` | 保持，行/块底色对齐 `#f0f0f0` |
| `el-menu`（管理后台） | 激活项由橙色改为黑字 + 灰底 |
| `el-switch` / `el-radio` / `el-checkbox` | 选中态改黑 |

## 6. 落地方式与文件清单

| 文件 | 改动 |
| --- | --- |
| `frontend/src/styles/main.css` | **主要改造**：重定义 `:root` 变量 + 重写全部组件规则块 + 追加 EP 覆盖段 |
| `frontend/index.html` | **同步重做骨架屏**：白底、发丝线、4px 圆角、灰条占位，使其与新风格一致（否则出现"旧风格骨架 → 新风格内容"的割裂） |
| `frontend/src/**/*.vue` | **不改模板**（含 `PostCard` 的 `💬`、`TopNav` 的 `🔍` 等 emoji，与已确认的稿子一致，本次保留） |

变量名（`--of-*`）**全部保留**，只改值，避免 20+ 文件连锁修改。

## 7. 非目标（YAGNI）

- 不改信息架构、不改路由、不改任何接口与后端。
- 不引入新字体文件/新依赖（继续用系统字体栈）。
- 不做深色模式（本次只做浅色）。
- 不做交互动效升级（滚动动画、骨架过渡等）。
- 不重做页面布局结构（列数、顺序保持），只换视觉语言。
- 不替换 Element Plus 组件库。

## 8. 风险与验证

| 风险 | 缓解 |
| --- | --- |
| 全局 CSS 影响未逐一设计的页面 | 已确认"全部统一"；实现后需逐一目视检查登录/注册/资料/认证/后台 |
| 去掉卡片后信息层级变弱 | 用字号/字重/发丝线三层手段补回层级；帖子标题加重到 700 |
| EP 组件颜色覆盖不全（部分用 SCSS 变量硬编码） | 以"实际渲染"为准，逐组件截图核对；必要时补选择器 |
| 骨架屏与内容风格不一致 | 骨架屏与 `main.css` 同批改，同批部署 |

**验收标准**
1. `npm run type-check` 与 `npm run build` 通过。
2. 三个目标页面 + 其他页面目视无"回归"（无残留橙色、无残留卡片阴影）。
3. 移动端断点（`≤900px`）侧栏堆叠正常。
4. 骨架屏首屏与新风格一致。
5. 主包体积不因本次改动回升。
