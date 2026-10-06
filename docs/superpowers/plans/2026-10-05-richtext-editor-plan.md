# Reddit 式双模式富文本编辑器 实现计划

- 日期：2026-10-05
- 契约来源：[2026-10-05-richtext-editor-design.md](../specs/2026-10-05-richtext-editor-design.md)
- 目标：发帖、评论、回复统一使用「富文本 / Markdown」双模式编辑器；Markdown 为唯一存储格式；展示端统一 Markdown 渲染并做 XSS 清洗。
- 范围：仅前端。后端不改表、不改接口。

> 每个阶段结束后执行该阶段的「验证」，通过后再进入下一阶段。

---

## Phase 1 依赖与基础渲染

### 1.1 安装依赖
- [ ] `frontend/` 安装：`@tiptap/vue-3`、`@tiptap/pm`、`@tiptap/starter-kit`、`@tiptap/markdown`、`@tiptap/extension-placeholder`、`marked`、`dompurify`（均取最新 v3 / 稳定版）。
- [ ] 安装后记录解析到的实际版本号到 `package.json`。

### 1.2 `utils/format.ts` 新增 `stripMarkdown`
- [ ] 导出 `stripMarkdown(md: string): string`：去除标题 `#`、强调 `*_~`、行内/围栏代码、链接语法（保留文本）、引用 `>`、列表符号、图片语法（丢弃或保留 alt）。
- [ ] 折叠连续空白为单空格并 `trim`，供摘要使用。

### 1.3 新增 `components/MarkdownContent.vue`
- [ ] props：`markdown: string`、可选 `class`。
- [ ] `computed`：`DOMPurify.sanitize(marked.parse(props.markdown || ''))`。
- [ ] 模板：`<div class="markdown-body" v-html="html" />`。
- [ ] 基础排版样式（标题、列表、引用、代码块、链接、段落间距）。

**验证**：`npm run type-check` 通过。

---

## Phase 2 双模式编辑器组件

### 2.1 编辑器内核
- [ ] 新增 `frontend/src/components/RichTextEditor.vue`。
- [ ] extensions：`StarterKit`、`Markdown`、`Placeholder.configure({ placeholder })`。
- [ ] `useEditor({ content, onUpdate })`；`onUpdate` 内 `emit('update:modelValue', editor.getMarkdown())`。
- [ ] props：`modelValue: string`、`placeholder?: string`、`compact?: boolean`、`minHeight?: number`。
- [ ] `onBeforeUnmount` 调用 `editor.value?.destroy()`。

### 2.2 双向同步（避免循环）
- [ ] 内部标志 `syncing`：`onUpdate` 写回时置位，跳过自身触发的 `watch`。
- [ ] `watch(() => props.modelValue)`：当外部值与 `editor.getMarkdown()` 不一致时，`editor.commands.setContent(value, { contentType: 'markdown' })`。

### 2.3 模式切换
- [ ] `mode: 'rich' | 'markdown'`，初值读 `localStorage['editor-mode']`，缺省 `'rich'`。
- [ ] 切换到 `markdown`：`mdText = editor.getMarkdown()` 填入 textarea。
- [ ] 切回 `rich`：`editor.commands.setContent(mdText, { contentType: 'markdown' })`。
- [ ] Markdown 模式编辑 textarea 时同步 `emit('update:modelValue', mdText)`。
- [ ] 切换时写入 localStorage。

### 2.4 富文本工具栏
- [ ] 按钮：加粗、斜体、删除线、行内代码、H1、H2、无序列表、有序列表、引用、代码块、链接、撤销、重做。
- [ ] 每按钮 `:class="{ active: editor.isActive(...) }"` 反映激活态；`compact` 时精简为 加粗/斜体/列表/链接/代码。
- [ ] 链接：`window.prompt` 输入 URL，`editor.chain().focus().setLink({ href }).run()`；已有链接时提供「移除链接」。

### 2.5 Markdown 模式
- [ ] `textarea`（`v-model="mdText"`）+ 「预览」开关。
- [ ] 预览复用 `MarkdownContent`。

**验证**：`npm run type-check` 通过；本地手测两模式互转内容不丢。

---

## Phase 3 接入点改造

### 3.1 发帖框 — [ForumPage.vue](file:///c:/Users/a7356/DesignCode/mall/openforum/frontend/src/views/ForumPage.vue)
- [ ] 将 `el-input type="textarea"`（content）替换为 `RichTextEditor v-model="createForm.content"`。
- [ ] 保留原有「非空」校验；清空逻辑改为 `createForm.content = ''`。

### 3.2 帖子详情 — [PostDetail.vue](file:///c:/Users/a7356/DesignCode/mall/openforum/frontend/src/views/PostDetail.vue)
- [ ] 正文 `{{ post.content }}` → `<MarkdownContent :markdown="post.content" />`。
- [ ] 评论输入框 → `RichTextEditor v-model="commentText" compact`。

### 3.3 评论树 — [CommentNode.vue](file:///c:/Users/a7356/DesignCode/mall/openforum/frontend/src/components/CommentNode.vue)
- [ ] 评论正文 `{{ comment.content }}` → `<MarkdownContent :markdown="comment.content" />`。
- [ ] 回复输入框 → `RichTextEditor v-model="replyText" compact`。
- [ ] 保持 `emit('reply', { parentId, content })` 的 content 为 Markdown。

### 3.4 列表摘要 — [PostCard.vue](file:///c:/Users/a7356/DesignCode/mall/openforum/frontend/src/components/PostCard.vue)
- [ ] `excerpt` 改用 `stripMarkdown(props.post.content)` 后再截断。

### 3.5 按需加载
- [ ] 在上述引用处使用 `defineAsyncComponent(() => import('@/components/RichTextEditor.vue'))`，避免拖累首屏。

**验证**：`npm run type-check` 通过。

---

## Phase 4 构建与验收

### 4.1 构建
- [ ] `npm run build` 通过；确认新 chunk 产出。

### 4.2 手动验收
- [ ] 发帖：默认富文本，工具栏加粗/列表/引用生效；切 Markdown 可见对应语法；切回富文本保留。
- [ ] Markdown 模式书写 `# 标题`/`**加粗**`/`- 列表`/`[链接](url)`，切富文本正确渲染。
- [ ] 提交后详情页正文按 Markdown 渲染正确。
- [ ] 顶层评论与回复：紧凑编辑器可用，提交后渲染正确。
- [ ] 列表摘要不含 Markdown 符号。
- [ ] XSS：提交 `<script>alert(1)</script>` 与 `<img onerror=...>`，渲染不执行、被清洗。
- [ ] 历史纯文本帖/评论渲染正常。

### 4.3 部署（可选，沿用既有流程）
- [ ] `frontend/dist` 打包上传至 `/mydata/nginx/html/forum`，`chmod -R a+rX`；旧目录备份。

---

## 风险与回滚

- `@tiptap/markdown` 为 Beta，高级语法往返可能丢失：仅承诺基础语法保真。
- 若编辑器导致构建/运行问题：回滚 `package.json` 与新增组件、恢复三处输入框为 `el-input` 即可，无后端/数据影响（已提交内容为 Markdown 纯文本，仍可正常渲染）。
