# Reddit 式编辑器对齐（布局 + 媒体上传）— 设计规格

- 日期：2026-10-06
- 状态：已评审通过，待实现
- 范围：前端（Vue 3 + TipTap v3）与后端（Spring Boot，新增媒体上传/读取接口）
- 前置规格：[2026-10-05-richtext-editor-design.md](file:///c:/Users/a7356/DesignCode/mall/openforum/docs/superpowers/specs/2026-10-05-richtext-editor-design.md)（双模式编辑器，已上线）

## 1. 背景与目标

上一版已实现 Markdown / 富文本双模式编辑器与 Markdown 作为唯一存储格式。本次要求编辑器 **UI 与工具集对齐 Reddit**：

1. 编辑器顶部为「模式标题 + 切换链接」的行（`Markdown Editor` / `Rich Text Editor` + `Switch to …`），而非当前的右上角胶囊按钮。
2. 格式化工具栏移到 **输入区下方**（Reddit 布局）。
3. 富文本模式工具栏补齐 Reddit 具备的能力：链接、图片、视频、加粗、斜体、删除线、上标、标题、无序/有序列表、引用、行内代码、代码块、表格。
4. 支持 **图片/视频上传** 并插入正文（GIF 本次不做）。
5. Markdown 模式底部工具栏提供「插入链接 / 图片 / 视频」。

非目标（YAGNI）：

- 不做 GIF（不上 Giphy）。
- 不做协同编辑、@提及、数学公式、投票/表情等。
- 不做媒体文件的编辑历史、回收站、配额统计。

## 2. 关键决策（已与用户确认）

| 决策点 | 结论 |
| --- | --- |
| 工具栏范围 | 完全对齐 Reddit（含图片/视频上传） |
| 图片存储 | 标准 Markdown `![alt](url)` |
| 视频存储 | 内嵌原始 HTML `<video src="…" controls></video>` 存入 Markdown |
| GIF | 本次不做，工具栏仅保留「图片」「视频」 |
| Markdown 模式「预览」按钮 | 移除（对齐 Reddit，无内联预览） |
| 媒体后端 | 新建 `post_media` 表 + 新接口（不复用认证媒体） |

## 3. 技术选型

| 用途 | 选型 | 说明 |
| --- | --- | --- |
| 图片节点 | `@tiptap/extension-image` | 与 `@tiptap/markdown` 互转为 `![alt](src)` |
| 表格 | `@tiptap/extension-table`（含 Row/Cell/Header） | TipTap v3 官方表格套件 |
| 上标 | `@tiptap/extension-superscript` | Reddit 的 X² |
| 视频节点 | 自定义 `Node`（基于 `@tiptap/core`） | 渲染 `['video', { src, controls: '' }]` |
| 媒体存储 | 复用 `StorageService`（MinIO 私有桶） | 生产 `MINIO_ENABLED=true` 已启用 |
| 展示端清洗 | `marked` + `dompurify`（既有） | 需放行 `video` 标签 |

理由：全部为 TipTap v3 官方扩展，和既有 `@tiptap/markdown` 存储格式一致；媒体存储复用现有 MinIO 能力，无新基础设施。

## 4. 编辑器 UI 设计（`RichTextEditor.vue` 重构）

### 4.1 结构

```
┌──────────────────────────────────────────────────────────┐
│ Markdown Editor ⓘ            Switch to Rich Text Editor   │  ← 头部行
├──────────────────────────────────────────────────────────┤
│  textarea（Markdown）  /  EditorContent（富文本）          │  ← 输入区
├──────────────────────────────────────────────────────────┤
│ 🔗 🖼 🎬 │ B i S X² T↑ │ •≡ 1≡ ❝ </> { } ⊞ ⋯             │  ← 底部工具栏
└──────────────────────────────────────────────────────────┘
```

### 4.2 头部行

- 左侧：模式名文本，`mode === 'markdown' ? 'Markdown Editor' : 'Rich Text Editor'`；后跟 `ⓘ` 图标（`el-tooltip` 展示一行使用说明）。
- 右侧：文字链接，`mode === 'rich' ? 'Switch to Markdown Editor' : 'Switch to Rich Text Editor'`，点击 `toggleMode()`。
- 样式沿用主题 token（`--of-primary`、`--of-text-muted` 等），与全站一致。

### 4.3 底部工具栏

- 通用按钮：图标 + `title` 提示，`@mousedown.prevent` 防止失焦。
- 富文本模式（`mode === 'rich'`）：
  - 插入组：链接、图片、视频
  - 格式组：B（加粗）、i（斜体）、S（删除线）、X²（上标）
  - 块组：标题（下拉，H1–H3）、无序列表、有序列表、引用（❝）、行内代码（`</>`）、代码块（`{ }`）、表格（⊞）
  - 激活态由 `editor.isActive(...)` 驱动（保留现有 `revision` 机制）。
- Markdown 模式（`mode === 'markdown'`）：
  - 仅插入组：链接、图片、视频。点击后在 textarea 光标处插入对应语法：
    - 链接：`[文本](url)`
    - 图片：`![alt](url)`
    - 视频：`<video src="url" controls></video>`
- `compact`（评论/回复）：精简集 = 加粗、斜体、无序列表、链接、图片、视频。

### 4.4 模式切换

- 维持现状：`openforum-editor-mode` localStorage 记忆；切换时富文本侧 `getMarkdown()` / `setContent(md, { contentType: 'markdown' })` 双向转换。
- 移除底部的「支持 Markdown 语法」提示条与「预览」按钮。

## 5. TipTap 扩展与 Markdown 序列化

- **图片**：`Image` 扩展；Markdown 互转 `![alt](src)`。
- **表格**：`Table` + `TableRow` + `TableCell` + `TableHeader`；Markdown 使用 GFM 表格语法。
- **上标**：`Superscript`；Markdown 序列化为 `<sup>…</sup>`。
- **视频（自定义节点）**：`name: 'video'`，`parseHTML` 匹配 `video[src]`，`renderHTML` 输出 `['video', { src, controls: '', poster? }]`；Markdown 序列化输出原始 HTML `<video src="…" controls></video>`，解析时识别该 HTML。

  **Phase 0 已验证通过**（Node 脚本实测）：在 Node 配置**顶层**提供 `markdownTokenizer`（`level: 'block'`，匹配 `^<video …></video>`）、`parseMarkdown`、`renderMarkdown` 三项即可实现双向转换；`<video>` → `{type:'video', attrs:{src}}` → 原样 HTML，往返稳定。

  ⚠️ 注意：`@tiptap/markdown` 运行时通过 `getExtensionField` 读取的是**顶层**字段；官方文档示例中的嵌套 `markdown: { … }` 键**不会被读取**，务必使用顶层写法。

  另注：`marked` 不把 `<video>` 识别为块级 HTML（会退化为段落文本），故此场景**必须**依赖上述自定义 tokenizer，不能只靠 `parseHTML`。

- **链接**：沿用 StarterKit 内置 `link`，`window.prompt` 输入 URL（既有实现）。

## 6. 后端媒体设计

### 6.1 实体

新增 `PostMedia`（`ddl-auto=update` 自动建表，无需迁移脚本）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | BIGINT PK | 自增 |
| userId | BIGINT | 上传者 |
| objectKey | VARCHAR | MinIO 对象键，前缀 `post/…` |
| contentType | VARCHAR | MIME |
| sizeBytes | BIGINT | 字节数 |
| createdAt | DATETIME | 创建时间 |

### 6.2 接口

| 方法 | 路径 | 鉴权 | 说明 |
| --- | --- | --- | --- |
| POST | `/api/media` | 需登录 | `multipart/form-data`，字段 `file`；返回 `{ id, contentType, sizeBytes }` |
| GET | `/api/media/{id}` | 公开 | 流式返回字节，带 `Content-Type` 与长缓存头 |

- 前端据 `id` 拼接 URL：`${BASE_URL}/media/${id}`（生产即 `/forum/api/media/{id}`）。
- 上传校验：图片 `image/jpeg|png|webp|gif`，视频 `video/mp4|webm`；大小沿用 `openforum.auth.max-image-bytes` / `max-video-bytes`。
- 存储：`StorageService.put("post/{userId}/{yyyyMMdd}/{uuid}.{ext}", …)`。

### 6.3 安全

- `SecurityConfig` 新增：`requestMatchers(HttpMethod.GET, "/media/**", "/api/media/**").permitAll()`；`POST /api/media` 因不在放行列表而默认要求认证。
- 上传实现内仍调用 `SecurityUtils.requireUser()` 取用户，双保险。
- 读取为只读流式，不暴露 objectKey。

## 7. 展示端渲染与安全

`MarkdownContent.vue` 的 DOMPurify 配置：在 `USE_PROFILES: { html: true }` 基础上显式放行：

- `video`：`src`、`controls`、`poster`、`width`、`height`、`preload`
- `img`：`src`、`alt`、`width`、`height`
- 明确**禁止** `script`、`iframe`、`on*` 事件（保持默认黑白名单）。

新增样式：正文内 `img` / `video` 最大宽度 100%、圆角、垂直间距，避免溢出。

## 8. 数据流

```
选择文件 → POST /api/media（登录）→ { id } → 前端拼 url
      → 富文本：插入 Image / Video 节点（或 textarea 插入语法）
      → 序列化为 Markdown（![alt](url) / <video …>）
      → 提交 post/comment content
      → 详情页 MarkdownContent：marked → DOMPurify → v-html（含 video）
```

## 9. 依赖变更

`frontend/package.json` dependencies 新增：

- `@tiptap/extension-image`
- `@tiptap/extension-table`
- `@tiptap/extension-superscript`

（`@tiptap/core`、`@tiptap/vue-3`、`@tiptap/pm`、`@tiptap/starter-kit`、`@tiptap/markdown` 已存在。）

后端无新依赖（MinIO 客户端已存在）。

## 10. 测试与验收

自动化：

- 后端：`PostMediaService` 单测——类型白名单、大小上限、未登录拒绝、公开读取返回正确 `Content-Type`。
- 前端：`npm run type-check` 与 `npm run build` 通过。

bsk 浏览器验收：

1. 发帖框：默认模式头部显示 `Markdown Editor` / `Switch to Rich Text Editor`；底部工具栏图标齐全。
2. 切到富文本：工具栏出现 B/i/S/X²/标题/列表/引用/代码/表格；激活态随选区变化。
3. 上传图片：工具栏插入图像，Markdown 模式下可见 `![alt](/forum/api/media/{id})`。
4. 上传视频：Markdown 含 `<video src=… controls>`；详情页可播放。
5. 表格、上标往返：富文本 ↔ Markdown 切换内容保真。
6. 详情页渲染：图片/视频/表格/上标正确显示，`overflow` 不溢出。
7. 历史纯文本/旧 Markdown 帖渲染不受影响。
8. 安全：含 `<script>`、`<img onerror>`、`<video onerror>` 的内容被清洗，事件不触发；`<iframe>` 被移除。

## 11. 风险与对策

| 风险 | 对策 |
| --- | --- |
| `@tiptap/markdown` 不支持自定义 Video 节点序列化 | 首步 spike 验证；不可行则视频退化为链接插入，并告知用户 |
| 新增扩展体积继续增大（编辑器 chunk 已 ~450KB） | 编辑器为异步组件，按需加载；新增扩展体积可控 |
| 公开媒体接口被滥用（盗链/大文件） | 仅允许白名单类型与大小；读取接口只读、长缓存；不暴露对象键 |
| GFM 表格与 `@tiptap/markdown` 往返有损 | 承诺基础表格；文档标注；复杂合并单元格不保证 |
| MinIO 生产桶 `openforum-auth` 名带 auth，语义不符 | 复用同一 bucket，仅以 `post/` 前缀区分；不改桶名避免影响既有认证媒体 |
