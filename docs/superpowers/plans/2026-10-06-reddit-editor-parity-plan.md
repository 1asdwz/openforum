# Reddit 式编辑器对齐（布局 + 媒体上传）实现计划

- 日期：2026-10-06
- 契约来源：[2026-10-06-reddit-editor-parity-design.md](../specs/2026-10-06-reddit-editor-parity-design.md)
- 目标：编辑器 UI/工具集对齐 Reddit（头部模式行 + 底部工具栏），支持图片/视频上传并插入正文；Markdown 仍为唯一存储格式。
- 范围：前端（Vue 3 + TipTap v3）+ 后端（Spring Boot，新增媒体上传/公开读取）。

> 每个阶段结束后执行该阶段的「验证」，通过后再进入下一阶段。

---

## Phase 0 技术验证（spike）

目的：确认 `@tiptap/markdown` 能否为**自定义 Video 节点**注册 Markdown 序列化/解析；决定视频方案。

- [x] 在 `frontend/` 建临时脚本或临时扩展，验证以下任一可行：
  - 自定义 `Node` 提供 `renderMarkdown` / `parseMarkdown`（或等价的 `Markdown` 扩展 handler）；
  - 或在 `Markdown` 扩展配置中注册节点级序列化器。
- [x] 若可行 → 采用「视频序列化为原始 HTML `<video src="…" controls></video>`」。
- [x] 若不可行 → 记录结论，视频降级为「以链接形式插入」，并**同步告知用户**后再继续。

**验证**：控制台演示：含 video 节点的文档 → `getMarkdown()` 输出预期字符串；再由该字符串 `setContent` 还原。

> 结论：可行。关键点是 `@tiptap/markdown` 只读取**顶层**扩展字段（`markdownTokenizer` / `parseMarkdown` / `renderMarkdown`），官方文档中的嵌套 `markdown: {…}` 写法无效。`<video>` 不被 marked 识别为块级 HTML，必须提供自定义 block tokenizer。

---

## Phase 1 后端媒体

### 1.1 实体与仓储
- [x] 新增 `entity/PostMedia.java`：`id`、`userId`、`objectKey`、`contentType`、`sizeBytes`、`createdAt`（`@PrePersist` 赋值）。表名 `post_media`（`ddl-auto=update` 自动建表）。
- [x] 新增 `repository/PostMediaRepository.java`（`JpaRepository<PostMedia, Long>`）。

### 1.2 服务 `service/PostMediaService.java`
- [x] `upload(MultipartFile file, Long userId)`：
  - 未登录 → `ApiException.unauthorized`；
  - 空文件 → `badRequest`；`storageService.isEnabled()` 为假 → `badRequest`；
  - 类型白名单：图片 `image/jpeg|png|webp|gif`，视频 `video/mp4|webm`；
  - 大小沿用 `openforum.auth.max-image-bytes` / `max-video-bytes`；
  - objectKey：`post/{userId}/{yyyyMMdd}/{uuid}.{ext}`；`storageService.put(...)`；保存实体。
- [x] `load(Long id)`：查实体 → `storageService.get(objectKey)`，返回 `record MediaResource(InputStream, contentType, size)`。

### 1.3 控制器 `controller/MediaController.java`
- [x] `POST /api/media`（`consumes = MULTIPART_FORM_DATA_VALUE`，`@RequestPart("file")`）→ `ApiResponse<MediaUploadResponse{id, contentType, sizeBytes}>`。
- [x] `GET /api/media/{id}` → `ResponseEntity<Resource>`，设置 `Content-Type`、`Content-Length`、`Cache-Control: public, max-age=31536000, immutable`。

### 1.4 安全 `security/SecurityConfig.java`
- [x] 新增：`requestMatchers(HttpMethod.GET, "/media/**", "/api/media/**").permitAll()`。
- [x] `POST /api/media` 保持需登录（不加入放行列表）；控制器内仍 `SecurityUtils.requireUser()`。

### 1.5 配置
- [x] 复用 `OpenForumAuthProperties` 的 `maxImageBytes` / `maxVideoBytes`（不新增配置键）。

**验证**：`mvn -q test` 通过；`PostMediaServiceTest`（类型白名单、超限、未登录拒绝、load 内容类型）通过。→ 9 个用例全绿。

---

## Phase 2 编辑器 UI 重构（`RichTextEditor.vue`）

### 2.1 头部行
- [x] 移除原顶部工具栏与右上角「Markdown」胶囊按钮。
- [x] 新增头部行：左侧 `{{ mode === 'rich' ? 'Rich Text Editor' : 'Markdown Editor' }}` + `el-tooltip` 信息图标；右侧文字链接 `Switch to Markdown Editor` / `Switch to Rich Text Editor` → `toggleMode()`。

### 2.2 底部工具栏
- [x] 将工具栏移到输入区**下方**（富文本模式与 Markdown 模式各一套，或共用容器按 mode 切换按钮集）。
- [x] 富文本按钮：链接、图片、视频、B、i、S、X²、（标题下拉 H1–H3）、无序列表、有序列表、引用、行内代码、代码块、表格。
- [x] 激活态继续用 `revision` + `editor.isActive(...)`。
- [x] Markdown 模式按钮：仅链接、图片、视频；点击后在 textarea 光标处插入 `[文本](url)` / `![alt](url)` / `<video src="url" controls></video>`。
- [x] `compact`：加粗、斜体、无序列表、链接、图片、视频。

### 2.3 清理
- [x] 删除「预览」按钮、「支持 Markdown 语法」提示条与相关样式/状态。
- [x] 保留 `openforum-editor-mode` localStorage 记忆与双向转换逻辑。

**验证**：`npm run type-check` 通过；本地手测：头部文案随模式变化、切换链接可点、Markdown 模式插入语法正确落位。

---

## Phase 3 媒体上传接入（前端）

### 3.1 API 与类型
- [x] `types/index.ts` 新增 `MediaUploadResult { id: number; contentType: string; sizeBytes: number }`。
- [x] `api/index.ts` 新增 `mediaApi`：`upload(file: File)` → `POST /media`（`multipart/form-data`）、`url(id)` → `${BASE_URL}/media/${id}`。

### 3.2 富文本插入
- [x] `@tiptap/extension-image`：工具栏「图片」→ 隐藏 `input[type=file][accept=image/*]` → `mediaApi.upload` → `editor.chain().focus().setImage({ src, alt }).run()`。
- [x] 自定义 Video 节点（Phase 0 结论）：工具栏「视频」→ `input[type=file][accept=video/*]` → 上传 → 插入 video 节点。
- [x] 上传中状态（禁用按钮/轻提示）；失败 `ElMessage.error`。

### 3.3 Markdown 模式插入
- [x] textarea 记录 `selectionStart/End`，插入语法并重新定位光标。

**验证**：本地（dev + 已启用 MinIO 或 mock 上传）插入图片/视频，Markdown 输出符合预期。

---

## Phase 4 新增格式扩展

- [x] `@tiptap/extension-table`（`Table` + `TableRow` + `TableCell` + `TableHeader`）注册；工具栏「表格」插入 3×3 空表并配置最小样式。
- [x] `@tiptap/extension-superscript` 注册；工具栏「X²」切换。
- [x] 标题下拉（`el-dropdown`）：普通 / H1 / H2 / H3，切换 `toggleHeading`。
- [x] 详情页样式：表格边框、`sup`、图片/视频最大宽度 100%。

**验证**：`npm run type-check` 通过；富文本 ↔ Markdown 往返：表格/上标/标题不丢内容。

> 补充：内置 Superscript 无 Markdown 支持，已 extend 补齐 `markdownTokenizer`(inline) + `parseMarkdown`(applyMark) + `renderMarkdown`。

---

## Phase 5 展示端渲染与安全

- [x] `MarkdownContent.vue` DOMPurify 配置：`USE_PROFILES: { html: true }` + 显式放行 `video`(`src`/`controls`/`poster`/`preload`) 与 `img`(`src`/`alt`)；确保 `script`/`iframe`/`on*` 仍被禁止。
- [x] 样式：`.markdown-body img, .markdown-body video { max-width: 100%; height: auto; border-radius: var(--of-radius-sm); }`，表格、`sup` 样式。
- [x] 校验：`marked` 对原始 `<video>` 原样输出，经 DOMPurify 保留。

**验证**：本地渲染含图片/视频/表格/上标的 Markdown；含 `<script>`、`<img onerror>`、`<video onerror>`、`<iframe>` 的内容被清洗。

---

## Phase 6 构建、测试与上线

### 6.1 构建
- [x] 后端 `mvn -q package`（使用 JDK 25 终端）；前端 `npm run build`。
- [x] 确认前端新增 chunk 产出（`RichTextEditor-*.js` / `.css`）。

### 6.2 部署
- [x] 前端 `dist` 上传 `/mydata/nginx/html/forum/`，`chmod -R a+rX`；先备份旧 `static`/`index.html`。
- [x] 后端：构建镜像 → 替换 `openforum-backend` 容器（保留既有 MinIO/AI 环境变量），确认 `ddl-auto=update` 已建 `post_media` 表。
- [x] 确认 nginx `client_max_body_size` ≥ 60m。

### 6.3 bsk 浏览器验收
- [x] 发帖框：头部 `Markdown Editor` / `Switch to Rich Text Editor`；底部工具栏齐全。
- [x] 切富文本：B/i/S/X²/标题/列表/引用/代码/表格可见，激活态随选区变化。
- [x] 上传图片：Markdown 模式插入 `![alt](…/api/media/{id})`；详情页显示（`naturalWidth=500`）。
- [x] 上传视频：Markdown 含 `<video src=… controls>`；详情页可播放（`readyState=4`、`duration≈10s`）。
- [x] 表格/上标往返保真。
- [x] XSS：`<script>`、`<img onerror>`、`<video onerror>`、`<iframe>`、`javascript:` 链接均被清洗，`window.__xss` 未定义（事件不触发）。
- [x] 历史纯文本/旧 Markdown 帖渲染不受影响；评论 compact 编辑器可用（`B i •≡ 🔗 🖼 🎬`）。

---

## 风险与回滚

- **视频自定义节点序列化**（Phase 0）：不可行则降级为链接插入并告知用户。→ 已验证可行。
- **编辑器体积**：新增表格/图片/上标扩展，chunk 继续增大；编辑器为异步组件按需加载，可接受。
- **公开媒体接口**：仅白名单类型/大小、只读、长缓存，不暴露对象键。
- **回滚**：前端恢复 `static.bak.*` / `index.html.bak.*`；后端回退上一镜像（`post_media` 表保留无害，旧代码不读取）。
- **兼容**：既有正文均为 Markdown 纯文本，新增媒体为增量能力，不改动既有字段与接口。
