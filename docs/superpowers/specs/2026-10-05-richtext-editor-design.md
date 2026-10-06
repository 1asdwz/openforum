# Reddit 式双模式富文本编辑器 — 设计规格

- 日期：2026-10-05
- 状态：已评审通过，待实现
- 范围：前端（Vue 3 + TipTap v3）。后端不涉及。

## 1. 背景与目标

当前项目的发帖、评论、回复均为纯文本 `el-input type="textarea"`，正文以纯文本插值渲染（无 `v-html`）。用户希望获得与 Reddit 一致的编辑体验：在 **富文本（所见即所得）** 与 **Markdown** 两种模式间自由切换，且切换时内容双向自动转换。

目标：

1. 发帖、顶层评论、评论回复三处输入统一使用新的双模式编辑器。
2. Markdown 作为**唯一存储格式**，复用现有 `content` 字段，后端不改表、不改接口。
3. 展示端统一按 Markdown 渲染，历史纯文本内容天然兼容。
4. 渲染路径必须经过 XSS 清洗。

非目标（YAGNI）：

- 不做图片/文件上传进正文（现有编辑器亦无）。
- 不做协同编辑、@提及、表格、数学公式等高级节点。
- 不做发帖/评论的“编辑历史”改造。

## 2. 技术选型

| 用途 | 选型 | 说明 |
| --- | --- | --- |
| 富文本内核 | `@tiptap/vue-3` + `@tiptap/pm` + `@tiptap/starter-kit` | TipTap v3，Vue 3 原生绑定；StarterKit 已内置加粗/斜体/删除线/代码/标题/列表/引用/代码块/链接/撤销重做等 |
| Markdown 互转 | `@tiptap/markdown`（官方，Beta） | `editor.getMarkdown()`；`editor.commands.setContent(md, { contentType: 'markdown' })` |
| 占位提示 | `@tiptap/extension-placeholder` | 空内容占位 |
| Markdown 渲染 | `marked` + `dompurify` | 展示端 `marked.parse` 后 `DOMPurify.sanitize` |

理由：社区包 `tiptap-markdown` 已停止维护；TipTap v3 官方 `@tiptap/markdown` 提供双向 Markdown 支持，与“Markdown 为存储规范”的目标一致。

## 3. 架构与数据流

- 编辑器的 `v-model` **始终是 Markdown 字符串**，作为规范源（single source of truth）。
- 富文本模式：TipTap 维护内部文档；`onUpdate` 时调用 `editor.storage.markdown.getMarkdown()` 写回 model。
- 模式切换：
  - 富文本 → Markdown：`getMarkdown()` → 填入 textarea。
  - Markdown → 富文本：`editor.commands.setContent(md, { contentType: 'markdown' })`。
  - 由于 model 始终是 Markdown，切换仅需一次“解析/序列化”，双向自动、不丢内容（不支持的高级语法会被丢弃，属可接受的有损转换）。
- 提交：沿用现有 post/comment API，直接发送 model 的 Markdown 字符串。
- 历史数据：现有 `content` 为纯文本，本身就是合法 Markdown，渲染与摘要逻辑无需迁移。

## 4. 组件设计

### 4.1 `RichTextEditor.vue`（新增）

- 位置：`frontend/src/components/RichTextEditor.vue`
- 入参（props）：
  - `modelValue: string` — Markdown 源
  - `placeholder?: string`
  - `compact?: boolean` — 评论/回复用的紧凑样式（更小高度、精简工具栏）
  - `minHeight?: number`
- 出参（emits）：`update:modelValue`
- 结构：
  - 顶部模式切换（富文本 / Markdown），记住上次选择（localStorage）。
  - 富文本模式：工具栏（加粗、斜体、删除线、行内代码、H1、H2、无序列表、有序列表、引用、代码块、链接、撤销、重做）+ `EditorContent`。工具栏按钮通过 `editor.isActive(...)` 反映激活态。
  - Markdown 模式：`textarea` + 「预览」切换；预览复用 `MarkdownContent`。
- 链接：使用 `window.prompt` 输入 URL（保持零额外 UI 依赖）。

### 4.2 `MarkdownContent.vue`（新增）

- 位置：`frontend/src/components/MarkdownContent.vue`
- 入参：`markdown: string`
- 渲染：`DOMPurify.sanitize(marked.parse(markdown))` 后 `v-html`。
- 通过 `computed` 缓存结果，避免重复解析。

### 4.3 接入点改造

| 文件 | 现状 | 改为 |
| --- | --- | --- |
| [ForumPage.vue](file:///c:/Users/a7356/DesignCode/mall/openforum/frontend/src/views/ForumPage.vue) 发帖框 | `el-input type="textarea"` | `RichTextEditor`（标准尺寸） |
| [PostDetail.vue](file:///c:/Users/a7356/DesignCode/mall/openforum/frontend/src/views/PostDetail.vue) 正文 | `{{ post.content }}` | `MarkdownContent` |
| [PostDetail.vue](file:///c:/Users/a7356/DesignCode/mall/openforum/frontend/src/views/PostDetail.vue) 评论框 | `el-input type="textarea"` | `RichTextEditor`（compact） |
| [CommentNode.vue](file:///c:/Users/a7356/DesignCode/mall/openforum/frontend/src/components/CommentNode.vue) 评论正文 | `{{ comment.content }}` | `MarkdownContent` |
| [CommentNode.vue](file:///c:/Users/a7356/DesignCode/mall/openforum/frontend/src/components/CommentNode.vue) 回复框 | `el-input type="textarea"` | `RichTextEditor`（compact） |
| [PostCard.vue](file:///c:/Users/a7356/DesignCode/mall/openforum/frontend/src/components/PostCard.vue) 摘要 | 纯文本截断 | 去除 Markdown 语法后截断（新增本地工具函数 `stripMarkdown`，置于 `utils/format.ts`） |

## 5. 渲染与安全

- 所有用户正文的 HTML 渲染路径均为 `marked` → `DOMPurify.sanitize` → `v-html`。
- DOMPurify 默认白名单已禁止 `<script>`、`<iframe>`、`<style>`、`on*` 事件等；无需额外配置，但显式使用默认配置并注明。
- TipTap 输出为结构化文档，不会引入脚本；风险主要来自 Markdown 中的原始 HTML，由 DOMPurify 兜底。
- 历史纯文本内容不会包含 HTML，渲染安全。

## 6. 依赖变更

`frontend/package.json` dependencies 新增：

- `@tiptap/vue-3`
- `@tiptap/pm`
- `@tiptap/starter-kit`
- `@tiptap/markdown`
- `@tiptap/extension-placeholder`
- `marked`
- `dompurify`

## 7. 后端

无改动。`Post.content` 与 `Comment.content` 均为 `LONGTEXT`，容量充足；`PostCreateRequest` / `CommentCreateRequest` 对 `content` 无长度上限，仅 `@NotBlank`。

## 8. 测试与验收

自动化：

- `npm run type-check` 通过。
- `npm run build` 通过（产出包含新编辑器 chunk）。

手动验收：

1. 发帖：默认富文本模式，使用工具栏加粗/列表/引用，切换到 Markdown 模式可见对应语法；切回富文本格式保留。
2. Markdown 模式直接书写 `# 标题`、`**加粗**`、`- 列表`、`[链接](url)`，切到富文本正确渲染。
3. 提交后，帖子详情页正文按 Markdown 正确渲染（标题、列表、代码块、链接）。
4. 顶层评论与评论回复：紧凑编辑器可用，提交后评论正文正确渲染。
5. 列表页摘要不显示 Markdown 符号。
6. 安全：提交含 `<script>alert(1)</script>`、`<img onerror=...>` 的内容，渲染时不执行、被清洗。
7. 历史纯文本帖/评论渲染正常（向后兼容）。

## 9. 风险与对策

| 风险 | 对策 |
| --- | --- |
| `@tiptap/markdown` 为 Beta，个别语法往返有损 | 仅承诺基础语法（标题/列表/引用/代码/链接/强调）双向保真；文档中标注 |
| 新增依赖体积较大 | 接受；编辑器组件按需加载（异步组件），避免影响首屏 |
| 富文本与 Markdown 模式切换导致内容被解析器改写 | model 始终为 Markdown，切换即“序列化/解析”一次；不额外做差异合并 |
