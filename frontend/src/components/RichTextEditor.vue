<template>
  <div class="rte" :class="{ 'rte-compact': compact }">
    <div class="rte-header">
      <div class="rte-header-left">
        <span class="rte-mode-title">{{ modeTitle }}</span>
        <el-tooltip :content="modeHint" placement="top">
          <span class="rte-info" aria-label="使用说明">ⓘ</span>
        </el-tooltip>
      </div>
      <button type="button" class="rte-switch" @click="toggleMode">{{ switchLabel }}</button>
    </div>

    <div v-show="mode === 'rich'" class="rte-editor-wrap" :style="{ minHeight: `${resolvedMinHeight}px` }">
      <EditorContent :editor="editor" class="rte-editor" />
    </div>

    <textarea
      ref="mdRef"
      v-show="mode === 'markdown'"
      v-model="mdText"
      class="rte-md-input"
      :style="{ minHeight: `${resolvedMinHeight}px` }"
      :placeholder="placeholder || '使用 Markdown 语法书写…'"
      @input="onMarkdownInput"
    />

    <div class="rte-toolbar">
      <template v-if="compact">
        <button
          v-for="btn in compactButtons"
          :key="btn.key"
          type="button"
          class="rte-btn"
          :class="{ active: btn.active() }"
          :title="btn.title"
          :disabled="isUploading(btn)"
          @mousedown.prevent
          @click="btn.run()"
        >
          <span v-html="btn.label" />
        </button>
      </template>
      <template v-else-if="mode === 'rich'">
        <button
          v-for="btn in richInsert"
          :key="btn.key"
          type="button"
          class="rte-btn"
          :class="{ active: btn.active() }"
          :title="btn.title"
          :disabled="isUploading(btn)"
          @mousedown.prevent
          @click="btn.run()"
        >
          <span v-html="btn.label" />
        </button>
        <span class="rte-sep" />
        <button
          v-for="btn in richFormat"
          :key="btn.key"
          type="button"
          class="rte-btn"
          :class="{ active: btn.active() }"
          :title="btn.title"
          @mousedown.prevent
          @click="btn.run()"
        >
          <span v-html="btn.label" />
        </button>
        <span class="rte-sep" />
        <el-dropdown trigger="click" @command="onHeadingCommand">
          <button
            type="button"
            class="rte-btn rte-heading-btn"
            :class="{ active: headingLevel > 0 }"
            title="标题"
            @mousedown.prevent
          >
            {{ headingLevel > 0 ? `H${headingLevel}` : 'T' }}
          </button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item :command="0">正文</el-dropdown-item>
              <el-dropdown-item :command="1">标题 1</el-dropdown-item>
              <el-dropdown-item :command="2">标题 2</el-dropdown-item>
              <el-dropdown-item :command="3">标题 3</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <button
          v-for="btn in richBlock"
          :key="btn.key"
          type="button"
          class="rte-btn"
          :class="{ active: btn.active() }"
          :title="btn.title"
          @mousedown.prevent
          @click="btn.run()"
        >
          <span v-html="btn.label" />
        </button>
      </template>
      <template v-else>
        <button
          v-for="btn in mdButtons"
          :key="btn.key"
          type="button"
          class="rte-btn"
          :class="{ active: btn.active() }"
          :title="btn.title"
          :disabled="isUploading(btn)"
          @mousedown.prevent
          @click="btn.run()"
        >
          <span v-html="btn.label" />
        </button>
      </template>
      <div class="rte-toolbar-spacer" />
      <span v-if="uploading" class="rte-uploading">上传中…</span>
    </div>

    <input
      ref="imageInputRef"
      type="file"
      class="rte-file"
      accept="image/jpeg,image/png,image/webp,image/gif"
      @change="onFilePicked($event, 'image')"
    />
    <input
      ref="videoInputRef"
      type="file"
      class="rte-file"
      accept="video/mp4,video/webm"
      @change="onFilePicked($event, 'video')"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { EditorContent, useEditor } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import { Markdown } from '@tiptap/markdown'
import { Placeholder } from '@tiptap/extension-placeholder'
import Image from '@tiptap/extension-image'
import { TableKit } from '@tiptap/extension-table'
import { ElMessage } from 'element-plus'
import { mediaApi } from '../api'
import { Superscript } from './editor/superscript'
import { Video } from './editor/video'

const props = withDefaults(
  defineProps<{
    modelValue: string
    placeholder?: string
    compact?: boolean
    minHeight?: number
  }>(),
  {
    placeholder: '',
    compact: false,
    minHeight: 0,
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const MODE_KEY = 'openforum-editor-mode'

function readMode(): 'rich' | 'markdown' {
  return localStorage.getItem(MODE_KEY) === 'markdown' ? 'markdown' : 'rich'
}

const mode = ref<'rich' | 'markdown'>(readMode())
const mdText = ref(props.modelValue || '')
const mdRef = ref<HTMLTextAreaElement | null>(null)
const imageInputRef = ref<HTMLInputElement | null>(null)
const videoInputRef = ref<HTMLInputElement | null>(null)
const uploading = ref(false)
// 每次事务递增，用于驱动工具栏激活态刷新
const revision = ref(0)

const resolvedMinHeight = computed(() => {
  if (props.minHeight) return props.minHeight
  return props.compact ? 90 : 180
})

const modeTitle = computed(() => (mode.value === 'rich' ? 'Rich Text Editor' : 'Markdown Editor'))
const switchLabel = computed(() =>
  mode.value === 'rich' ? 'Switch to Markdown Editor' : 'Switch to Rich Text Editor'
)
const modeHint = computed(() =>
  mode.value === 'rich'
    ? '使用工具栏格式化内容，保存为 Markdown 格式'
    : '使用 Markdown 语法书写，支持插入图片与视频'
)

const editor = useEditor({
  content: props.modelValue || '',
  contentType: 'markdown',
  extensions: [
    StarterKit.configure({
      link: { openOnClick: false, autolink: true },
    }),
    Markdown,
    Placeholder.configure({
      placeholder: () => props.placeholder || '写点什么…',
    }),
    Image.configure({ inline: false, allowBase64: false }),
    TableKit,
    Superscript,
    Video,
  ],
  editorProps: {
    attributes: { class: 'rte-content' },
  },
  onUpdate: ({ editor: ed }) => {
    const md = ed.getMarkdown()
    mdText.value = md
    emit('update:modelValue', md)
  },
  onTransaction: () => {
    revision.value += 1
  },
})

watch(
  () => props.modelValue,
  (value) => {
    const next = value || ''
    if (mode.value === 'rich') {
      const ed = editor.value
      if (ed && next !== ed.getMarkdown()) {
        ed.commands.setContent(next, { contentType: 'markdown' })
      }
    }
    mdText.value = next
  }
)

function onMarkdownInput() {
  emit('update:modelValue', mdText.value)
}

function toggleMode() {
  if (mode.value === 'rich') {
    const ed = editor.value
    if (ed) mdText.value = ed.getMarkdown()
    mode.value = 'markdown'
  } else {
    const ed = editor.value
    if (ed) {
      nextTick(() => ed.commands.setContent(mdText.value, { contentType: 'markdown' }))
    }
    mode.value = 'rich'
  }
  localStorage.setItem(MODE_KEY, mode.value)
}

const headingLevel = computed(() => {
  void revision.value
  const ed = editor.value
  if (!ed) return 0
  for (const level of [1, 2, 3] as const) {
    if (ed.isActive('heading', { level })) return level
  }
  return 0
})

function onHeadingCommand(level: number) {
  const ed = editor.value
  if (!ed) return
  if (!level) {
    ed.chain().focus().setParagraph().run()
    return
  }
  ed.chain().focus().toggleHeading({ level: level as 1 | 2 | 3 }).run()
}

function setLink() {
  const ed = editor.value
  if (!ed) return
  const prev = (ed.getAttributes('link').href as string) || ''
  const url = window.prompt('链接地址', prev || 'https://')
  if (url === null) return
  const trimmed = url.trim()
  if (!trimmed) {
    ed.chain().focus().extendMarkRange('link').unsetLink().run()
    return
  }
  ed.chain().focus().extendMarkRange('link').setLink({ href: trimmed }).run()
}

function insertMarkdown(snippet: string) {
  const ta = mdRef.value
  if (!ta) {
    mdText.value += snippet
    emit('update:modelValue', mdText.value)
    return
  }
  const start = ta.selectionStart ?? mdText.value.length
  const end = ta.selectionEnd ?? start
  const next = mdText.value.slice(0, start) + snippet + mdText.value.slice(end)
  mdText.value = next
  emit('update:modelValue', next)
  nextTick(() => {
    ta.focus()
    const pos = start + snippet.length
    ta.setSelectionRange(pos, pos)
  })
}

function onLinkButton() {
  if (mode.value === 'markdown') {
    const url = window.prompt('链接地址', 'https://')
    if (url === null) return
    const trimmed = url.trim()
    if (!trimmed) return
    const ta = mdRef.value
    const text = ta ? mdText.value.slice(ta.selectionStart ?? 0, ta.selectionEnd ?? 0) : ''
    insertMarkdown(`[${text || '文本'}](${trimmed})`)
    return
  }
  setLink()
}

function pickImage() {
  if (!uploading.value) imageInputRef.value?.click()
}

function pickVideo() {
  if (!uploading.value) videoInputRef.value?.click()
}

async function onFilePicked(event: Event, kind: 'image' | 'video') {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return

  uploading.value = true
  try {
    const result = await mediaApi.upload(file)
    const url = mediaApi.url(result.id)
    if (mode.value === 'rich') {
      const ed = editor.value
      if (!ed) return
      if (kind === 'image') {
        ed.chain().focus().setImage({ src: url, alt: file.name }).run()
      } else {
        ed.chain().focus().setVideo({ src: url }).run()
      }
    } else if (kind === 'image') {
      const name = file.name.replace(/\.[^.]+$/, '')
      insertMarkdown(`![${name}](${url})`)
    } else {
      insertMarkdown(`<video src="${url}" controls></video>`)
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : '上传失败')
  } finally {
    uploading.value = false
  }
}

interface ToolbarButton {
  key: string
  label: string
  title: string
  active: () => boolean
  run: () => void
}

const LABELS: Record<string, string> = {
  link: '🔗',
  image: '🖼',
  video: '🎬',
  bold: '<b>B</b>',
  italic: '<i>i</i>',
  strike: '<s>S</s>',
  superscript: 'X²',
  bulletList: '•≡',
  orderedList: '1≡',
  blockquote: '❝',
  code: '&lt;/&gt;',
  codeBlock: '{ }',
  table: '⊞',
}

function isUploading(btn: ToolbarButton) {
  return uploading.value && (btn.key === 'image' || btn.key === 'video')
}

const buttonMap = computed<Record<string, ToolbarButton>>(() => {
  const ed = editor.value
  void revision.value

  const map: Record<string, ToolbarButton> = {}

  map.link = {
    key: 'link',
    label: LABELS.link,
    title: '链接',
    active: () => (ed ? ed.isActive('link') : false),
    run: onLinkButton,
  }
  map.image = {
    key: 'image',
    label: LABELS.image,
    title: '图片',
    active: () => false,
    run: pickImage,
  }
  map.video = {
    key: 'video',
    label: LABELS.video,
    title: '视频',
    active: () => false,
    run: pickVideo,
  }

  if (ed) {
    map.bold = {
      key: 'bold',
      label: LABELS.bold,
      title: '加粗',
      active: () => ed.isActive('bold'),
      run: () => ed.chain().focus().toggleBold().run(),
    }
    map.italic = {
      key: 'italic',
      label: LABELS.italic,
      title: '斜体',
      active: () => ed.isActive('italic'),
      run: () => ed.chain().focus().toggleItalic().run(),
    }
    map.strike = {
      key: 'strike',
      label: LABELS.strike,
      title: '删除线',
      active: () => ed.isActive('strike'),
      run: () => ed.chain().focus().toggleStrike().run(),
    }
    map.superscript = {
      key: 'superscript',
      label: LABELS.superscript,
      title: '上标',
      active: () => ed.isActive('superscript'),
      run: () => ed.chain().focus().toggleSuperscript().run(),
    }
    map.bulletList = {
      key: 'bulletList',
      label: LABELS.bulletList,
      title: '无序列表',
      active: () => ed.isActive('bulletList'),
      run: () => ed.chain().focus().toggleBulletList().run(),
    }
    map.orderedList = {
      key: 'orderedList',
      label: LABELS.orderedList,
      title: '有序列表',
      active: () => ed.isActive('orderedList'),
      run: () => ed.chain().focus().toggleOrderedList().run(),
    }
    map.blockquote = {
      key: 'blockquote',
      label: LABELS.blockquote,
      title: '引用',
      active: () => ed.isActive('blockquote'),
      run: () => ed.chain().focus().toggleBlockquote().run(),
    }
    map.code = {
      key: 'code',
      label: LABELS.code,
      title: '行内代码',
      active: () => ed.isActive('code'),
      run: () => ed.chain().focus().toggleCode().run(),
    }
    map.codeBlock = {
      key: 'codeBlock',
      label: LABELS.codeBlock,
      title: '代码块',
      active: () => ed.isActive('codeBlock'),
      run: () => ed.chain().focus().toggleCodeBlock().run(),
    }
    map.table = {
      key: 'table',
      label: LABELS.table,
      title: '表格',
      active: () => ed.isActive('table'),
      run: () => ed.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
    }
  }

  return map
})

function pick(keys: string[]) {
  return keys.map((key) => buttonMap.value[key]).filter((btn): btn is ToolbarButton => !!btn)
}

const richInsert = computed(() => pick(['link', 'image', 'video']))
const richFormat = computed(() => pick(['bold', 'italic', 'strike', 'superscript']))
const richBlock = computed(() =>
  pick(['bulletList', 'orderedList', 'blockquote', 'code', 'codeBlock', 'table'])
)
const mdButtons = computed(() => pick(['link', 'image', 'video']))
const compactButtons = computed(() =>
  mode.value === 'rich'
    ? pick(['bold', 'italic', 'bulletList', 'link', 'image', 'video'])
    : mdButtons.value
)

onBeforeUnmount(() => {
  editor.value?.destroy()
})
</script>

<style scoped>
.rte {
  border: 1px solid var(--of-border);
  border-radius: var(--of-radius-sm);
  background: var(--of-card);
  overflow: hidden;
}

.rte:focus-within {
  border-color: var(--of-primary);
}

.rte-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 6px 10px;
  border-bottom: 1px solid var(--of-border-soft);
  background: var(--of-bg);
}

.rte-header-left {
  display: flex;
  align-items: center;
  gap: 5px;
  min-width: 0;
}

.rte-mode-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--of-text);
}

.rte-info {
  font-size: 12px;
  color: var(--of-text-subtle);
  cursor: help;
}

.rte-switch {
  flex-shrink: 0;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--of-primary);
  font-size: 12px;
  cursor: pointer;
}

.rte-switch:hover {
  text-decoration: underline;
}

.rte-toolbar {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-wrap: wrap;
  padding: 4px 6px;
  border-top: 1px solid var(--of-border-soft);
  background: var(--of-bg);
}

.rte-toolbar-spacer {
  flex: 1;
}

.rte-sep {
  width: 1px;
  height: 18px;
  margin: 0 4px;
  background: var(--of-border);
}

.rte-btn {
  min-width: 28px;
  height: 28px;
  padding: 0 6px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--of-text);
  font-size: 13px;
  line-height: 1;
  cursor: pointer;
}

.rte-btn:hover:not(:disabled) {
  background: var(--of-border-soft);
}

.rte-btn.active {
  background: var(--of-primary-soft);
  color: var(--of-primary);
}

.rte-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.rte-heading-btn {
  min-width: 32px;
  font-weight: 600;
}

.rte-uploading {
  font-size: 12px;
  color: var(--of-text-muted);
}

.rte-file {
  display: none;
}

.rte-editor-wrap {
  padding: 10px 12px;
  cursor: text;
}

.rte-editor :deep(.rte-content) {
  outline: none;
  font-size: 15px;
  line-height: 1.7;
  word-break: break-word;
}

.rte-editor :deep(.rte-content p) {
  margin: 0 0 10px;
}

.rte-editor :deep(.rte-content > :last-child) {
  margin-bottom: 0;
}

.rte-editor :deep(.rte-content p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  height: 0;
  color: var(--of-text-subtle);
  pointer-events: none;
}

.rte-editor :deep(.rte-content blockquote) {
  margin: 0 0 10px;
  padding: 4px 14px;
  border-left: 3px solid var(--of-border);
  color: var(--of-text-muted);
}

.rte-editor :deep(.rte-content pre) {
  margin: 0 0 10px;
  padding: 12px 14px;
  border-radius: var(--of-radius-sm);
  background: #f6f8fa;
  overflow-x: auto;
}

.rte-editor :deep(.rte-content code) {
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--of-border-soft);
  font-family: 'SFMono-Regular', Consolas, Menlo, monospace;
  font-size: 0.9em;
}

.rte-editor :deep(.rte-content pre code) {
  padding: 0;
  background: transparent;
}

.rte-editor :deep(.rte-content a) {
  color: var(--of-primary);
  text-decoration: underline;
}

.rte-editor :deep(.rte-content ul),
.rte-editor :deep(.rte-content ol) {
  margin: 0 0 10px;
  padding-left: 24px;
}

.rte-editor :deep(.rte-content img),
.rte-editor :deep(.rte-content video) {
  max-width: 100%;
  height: auto;
  border-radius: var(--of-radius-sm);
}

.rte-editor :deep(.rte-content table) {
  width: 100%;
  margin: 0 0 10px;
  border-collapse: collapse;
}

.rte-editor :deep(.rte-content th),
.rte-editor :deep(.rte-content td) {
  padding: 6px 10px;
  border: 1px solid var(--of-border);
  text-align: left;
  vertical-align: top;
}

.rte-editor :deep(.rte-content th) {
  background: var(--of-border-soft);
  font-weight: 600;
}

.rte-md-input {
  display: block;
  width: 100%;
  padding: 10px 12px;
  border: none;
  outline: none;
  resize: vertical;
  font-family: 'SFMono-Regular', Consolas, Menlo, monospace;
  font-size: 13px;
  line-height: 1.6;
  color: var(--of-text);
}
</style>
