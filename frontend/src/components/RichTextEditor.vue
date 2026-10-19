<template>
  <div class="rte" :class="{ 'rte-compact': compact }">
    <div class="rte-bar">
      <template v-if="mode === 'rich' && !compact">
        <button
          v-for="btn in pick(['undo', 'redo'])"
          :key="btn.key"
          type="button"
          class="rte-btn"
          :title="btn.title"
          :disabled="btn.disabled ? btn.disabled() : false"
          @mousedown.prevent
          @click="btn.run()"
        >
          <span class="rte-icon" v-html="btn.icon" />
        </button>
        <span class="rte-sep" />

        <el-dropdown trigger="click" @command="onHeadingCommand">
          <button
            type="button"
            class="rte-btn rte-heading-btn"
            :class="{ active: headingLevel > 0 }"
            :title="headingLevel > 0 ? `Heading ${headingLevel}` : 'Paragraph'"
            @mousedown.prevent
          >
            <span class="rte-icon" v-html="ICONS.heading" />
          </button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item :command="0">Paragraph</el-dropdown-item>
              <el-dropdown-item :command="1">Heading 1</el-dropdown-item>
              <el-dropdown-item :command="2">Heading 2</el-dropdown-item>
              <el-dropdown-item :command="3">Heading 3</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <button
          v-for="btn in pick(['bold', 'italic', 'strike', 'superscript', 'code'])"
          :key="btn.key"
          type="button"
          class="rte-btn"
          :class="{ active: btn.active ? btn.active() : false }"
          :title="btn.title"
          @mousedown.prevent
          @click="btn.run()"
        >
          <span class="rte-icon" v-html="btn.icon" />
        </button>
        <span class="rte-sep" />

        <button
          v-for="btn in pick(['bulletList', 'orderedList', 'blockquote', 'codeBlock'])"
          :key="btn.key"
          type="button"
          class="rte-btn"
          :class="{ active: btn.active ? btn.active() : false }"
          :title="btn.title"
          @mousedown.prevent
          @click="btn.run()"
        >
          <span class="rte-icon" v-html="btn.icon" />
        </button>
        <span class="rte-sep" />
      </template>

      <template v-else-if="mode === 'rich' && compact">
        <button
          v-for="btn in pick(['bold', 'italic'])"
          :key="btn.key"
          type="button"
          class="rte-btn"
          :class="{ active: btn.active ? btn.active() : false }"
          :title="btn.title"
          @mousedown.prevent
          @click="btn.run()"
        >
          <span class="rte-icon" v-html="btn.icon" />
        </button>
        <span class="rte-sep" />
      </template>

      <button
        v-for="btn in pick(['link', 'image', 'video'])"
        :key="btn.key"
        type="button"
        class="rte-btn"
        :class="{ active: linkOpen && btn.key === 'link' }"
        :title="btn.title"
        :disabled="isUploading(btn)"
        @mousedown.prevent
        @click="btn.run()"
      >
        <span class="rte-icon" v-html="btn.icon" />
      </button>

      <el-dropdown v-if="mode === 'rich' && !compact" trigger="click" @command="onTableCommand">
        <button type="button" class="rte-btn" title="Table" @mousedown.prevent>
          <span class="rte-icon" v-html="ICONS.table" />
        </button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="insert">Insert table 3×3</el-dropdown-item>
            <el-dropdown-item command="rowBefore" :disabled="!inTable" divided>Insert row above</el-dropdown-item>
            <el-dropdown-item command="rowAfter" :disabled="!inTable">Insert row below</el-dropdown-item>
            <el-dropdown-item command="deleteRow" :disabled="!inTable">Delete row</el-dropdown-item>
            <el-dropdown-item command="colBefore" :disabled="!inTable" divided>Insert column left</el-dropdown-item>
            <el-dropdown-item command="colAfter" :disabled="!inTable">Insert column right</el-dropdown-item>
            <el-dropdown-item command="deleteCol" :disabled="!inTable">Delete column</el-dropdown-item>
            <el-dropdown-item command="deleteTable" :disabled="!inTable" divided>Delete table</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

      <div class="rte-bar-spacer" />

      <span v-if="uploading" class="rte-uploading">Uploading…</span>
      <span v-else class="rte-count">{{ charCount }} chars</span>

      <el-tooltip :content="modeHint" placement="top">
        <span class="rte-info" aria-label="Usage tips">
          <span class="rte-icon" v-html="ICONS.info" />
        </span>
      </el-tooltip>
      <button type="button" class="rte-switch" @click="toggleMode">{{ switchLabel }}</button>
    </div>

    <div v-if="linkOpen" class="rte-linkbar">
      <input
        ref="linkInputRef"
        v-model="linkValue"
        class="rte-link-input"
        type="text"
        placeholder="https:// or paste a link"
        @keydown.enter.prevent="confirmLink"
        @keydown.esc.prevent="closeLink"
      />
      <button type="button" class="rte-link-btn primary" @mousedown.prevent @click="confirmLink">Confirm</button>
      <button
        v-if="mode === 'rich' && linkExisting"
        type="button"
        class="rte-link-btn"
        @mousedown.prevent
        @click="removeLink"
      >
        Remove
      </button>
      <button type="button" class="rte-link-btn" @mousedown.prevent @click="closeLink">Cancel</button>
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
      :placeholder="placeholder || 'Write using Markdown…'"
      @input="onMarkdownInput"
      @paste="onMarkdownPaste"
      @dragover.prevent
      @drop.prevent="onMarkdownDrop"
    />

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
import { EditorContent, useEditor, type Editor } from '@tiptap/vue-3'
import { TextSelection } from '@tiptap/pm/state'
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
const linkInputRef = ref<HTMLInputElement | null>(null)
const uploading = ref(false)
// Incremented on every transaction to drive toolbar active-state refreshes
const revision = ref(0)

const linkOpen = ref(false)
const linkValue = ref('')
const linkExisting = ref(false)

const resolvedMinHeight = computed(() => {
  if (props.minHeight) return props.minHeight
  return props.compact ? 90 : 180
})

const switchLabel = computed(() =>
  mode.value === 'rich' ? 'Switch to Markdown' : 'Switch to rich text'
)
const modeHint = computed(() =>
  mode.value === 'rich'
    ? 'Format content with the toolbar; saved as Markdown'
    : 'Write in Markdown; supports inserting images and videos'
)

const charCount = computed(() => {
  void revision.value
  if (mode.value === 'rich') {
    const ed = editor.value
    if (ed) return ed.getText().replace(/\n/g, '').trim().length
  }
  return mdText.value.trim().length
})

const ICONS: Record<string, string> = {
  undo: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/></svg>',
  redo: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M21 7v6h-6"/><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3L21 13"/></svg>',
  bold: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M6 12h9a4 4 0 0 1 0 8H6V4h8a4 4 0 0 1 0 8"/></svg>',
  italic: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="4" x2="10" y2="4"/><line x1="14" y1="20" x2="5" y2="20"/><line x1="15" y1="4" x2="9" y2="20"/></svg>',
  strike: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M16 4H9a3 3 0 0 0-2.83 4"/><path d="M14 12a4 4 0 0 1 0 8H6"/><line x1="4" y1="12" x2="20" y2="12"/></svg>',
  superscript: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="m4 19 8-8"/><path d="m12 19-8-8"/><path d="M20 12h-4c0-1.5.44-2 1.5-2.5S20 8.33 20 7a2 2 0 0 0-3.5-1.3"/></svg>',
  link: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>',
  image: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.1-3.1a2 2 0 0 0-2.8 0L6 21"/></svg>',
  video: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="m22 8-6 4 6 4V8Z"/><rect x="2" y="6" width="14" height="12" rx="2"/></svg>',
  heading: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M6 12h12"/><path d="M6 20V4"/><path d="M18 20V4"/></svg>',
  bulletList: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><line x1="9" y1="6" x2="20" y2="6"/><line x1="9" y1="12" x2="20" y2="12"/><line x1="9" y1="18" x2="20" y2="18"/><circle cx="4.5" cy="6" r="1.2" fill="currentColor" stroke="none"/><circle cx="4.5" cy="12" r="1.2" fill="currentColor" stroke="none"/><circle cx="4.5" cy="18" r="1.2" fill="currentColor" stroke="none"/></svg>',
  orderedList: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><line x1="10" y1="6" x2="20" y2="6"/><line x1="10" y1="12" x2="20" y2="12"/><line x1="10" y1="18" x2="20" y2="18"/><path d="M4 6h1v4"/><path d="M4 10h2"/><path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1"/></svg>',
  blockquote: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h13"/><path d="M3 12h10"/><path d="M3 18h10"/><path d="M20 6v12"/></svg>',
  code: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>',
  codeBlock: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="m10 9-2 3 2 3"/><path d="m14 9 2 3-2 3"/></svg>',
  table: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/><path d="M3 15h18"/><path d="M12 3v18"/></svg>',
  info: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>',
}

const editor = useEditor({
  content: props.modelValue || '',
  contentType: 'markdown',
  extensions: [
    StarterKit.configure({
      link: { openOnClick: false, autolink: true },
    }),
    Markdown,
    Placeholder.configure({
      placeholder: () => props.placeholder || 'Write something…',
    }),
    Image.configure({ inline: false, allowBase64: false }),
    TableKit,
    Superscript,
    Video,
  ],
  editorProps: {
    attributes: { class: 'rte-content' },
    handlePaste: (_view, event) => {
      const media = mediaFiles(event.clipboardData?.files)
      if (!media.length) return false
      event.preventDefault()
      return handleMediaFiles(media)
    },
    handleDrop: (_view, event, _slice, moved) => {
      if (moved) return false
      const media = mediaFiles(event.dataTransfer?.files)
      if (!media.length) return false
      event.preventDefault()
      return handleMediaFiles(media)
    },
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
  closeLink()
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

const inTable = computed(() => {
  void revision.value
  return editor.value ? editor.value.isActive('table') : false
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

function onTableCommand(cmd: string) {
  const ed = editor.value
  if (!ed) return
  const c = ed.chain().focus()
  switch (cmd) {
    case 'insert':
      c.insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
      break
    case 'rowBefore':
      c.addRowBefore().run()
      break
    case 'rowAfter':
      c.addRowAfter().run()
      break
    case 'deleteRow':
      c.deleteRow().run()
      break
    case 'colBefore':
      c.addColumnBefore().run()
      break
    case 'colAfter':
      c.addColumnAfter().run()
      break
    case 'deleteCol':
      c.deleteColumn().run()
      break
    case 'deleteTable':
      c.deleteTable().run()
      break
  }
}

function openLink() {
  const ed = editor.value
  if (mode.value === 'rich' && ed) {
    linkValue.value = (ed.getAttributes('link').href as string) || ''
    linkExisting.value = ed.isActive('link')
  } else {
    linkValue.value = ''
    linkExisting.value = false
  }
  linkOpen.value = true
  nextTick(() => {
    const el = linkInputRef.value
    if (el) {
      el.focus()
      el.select()
    }
  })
}

function closeLink() {
  linkOpen.value = false
  linkValue.value = ''
  linkExisting.value = false
}

function confirmLink() {
  const trimmed = linkValue.value.trim()
  if (mode.value === 'rich') {
    const ed = editor.value
    if (ed) {
      if (!trimmed) {
        ed.chain().focus().extendMarkRange('link').unsetLink().run()
      } else if (ed.state.selection.empty && !ed.isActive('link')) {
        ed.chain()
          .focus()
          .insertContent({ type: 'text', text: trimmed, marks: [{ type: 'link', attrs: { href: trimmed } }] })
          .run()
      } else {
        ed.chain().focus().extendMarkRange('link').setLink({ href: trimmed }).run()
      }
    }
    closeLink()
    return
  }
  if (!trimmed) {
    closeLink()
    return
  }
  const ta = mdRef.value
  const text = ta ? mdText.value.slice(ta.selectionStart ?? 0, ta.selectionEnd ?? 0) : ''
  insertMarkdown(`[${text || trimmed}](${trimmed})`)
  closeLink()
}

function removeLink() {
  const ed = editor.value
  if (!ed) return
  ed.chain().focus().extendMarkRange('link').unsetLink().run()
  closeLink()
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
  if (linkOpen.value) {
    closeLink()
    return
  }
  openLink()
}

function pickImage() {
  if (!uploading.value) imageInputRef.value?.click()
}

function pickVideo() {
  if (!uploading.value) videoInputRef.value?.click()
}

/**
 * After inserting an image/video, tiptap leaves a node selection, so inserting again would overwrite the previous one.
 * Collapse the cursor after the inserted content so consecutive inserts (multi-image paste/drag) append one by one.
 */
function collapseSelection(ed: Editor) {
  const { state, view } = ed
  const next = TextSelection.near(state.doc.resolve(state.selection.to), 1)
  view.dispatch(state.tr.setSelection(next))
}

async function uploadAndInsert(file: File, kind: 'image' | 'video') {
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
      collapseSelection(ed)
    } else if (kind === 'image') {
      const name = file.name.replace(/\.[^.]+$/, '')
      insertMarkdown(`![${name}](${url})`)
    } else {
      insertMarkdown(`<video src="${url}" controls></video>`)
    }
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : 'Upload failed')
  } finally {
    uploading.value = false
  }
}

function mediaFiles(files: FileList | File[] | null | undefined): File[] {
  if (!files) return []
  return Array.from(files).filter(
    (f) => f.type.startsWith('image/') || f.type.startsWith('video/')
  )
}

function kindOf(file: File): 'image' | 'video' {
  return file.type.startsWith('video/') ? 'video' : 'image'
}

/** Upload and insert media in sequence (used for paste/drag) */
function handleMediaFiles(files: File[]): boolean {
  if (!files.length) return false
  void (async () => {
    for (const f of files) await uploadAndInsert(f, kindOf(f))
  })()
  return true
}

async function onFilePicked(event: Event, kind: 'image' | 'video') {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  await uploadAndInsert(file, kind)
}

function onMarkdownPaste(event: ClipboardEvent) {
  const media = mediaFiles(event.clipboardData?.files)
  if (!media.length) return
  event.preventDefault()
  handleMediaFiles(media)
}

function onMarkdownDrop(event: DragEvent) {
  const media = mediaFiles(event.dataTransfer?.files)
  if (!media.length) return
  handleMediaFiles(media)
}

interface ToolbarButton {
  key: string
  icon: string
  title: string
  active?: () => boolean
  disabled?: () => boolean
  run: () => void
}

function isUploading(btn: ToolbarButton) {
  return uploading.value && (btn.key === 'image' || btn.key === 'video')
}

const buttonMap = computed<Record<string, ToolbarButton>>(() => {
  const ed = editor.value
  void revision.value

  const map: Record<string, ToolbarButton> = {}

  map.undo = {
    key: 'undo',
    icon: ICONS.undo,
    title: 'Undo (Ctrl+Z)',
    disabled: () => (ed ? !ed.can().undo() : true),
    run: () => ed?.chain().focus().undo().run(),
  }
  map.redo = {
    key: 'redo',
    icon: ICONS.redo,
    title: 'Redo (Ctrl+Shift+Z)',
    disabled: () => (ed ? !ed.can().redo() : true),
    run: () => ed?.chain().focus().redo().run(),
  }
  map.link = {
    key: 'link',
    icon: ICONS.link,
    title: 'Link (Ctrl+K)',
    run: onLinkButton,
  }
  map.image = {
    key: 'image',
    icon: ICONS.image,
    title: 'Image',
    run: pickImage,
  }
  map.video = {
    key: 'video',
    icon: ICONS.video,
    title: 'Video',
    run: pickVideo,
  }

  if (ed) {
    map.bold = {
      key: 'bold',
      icon: ICONS.bold,
      title: 'Bold (Ctrl+B)',
      active: () => ed.isActive('bold'),
      run: () => ed.chain().focus().toggleBold().run(),
    }
    map.italic = {
      key: 'italic',
      icon: ICONS.italic,
      title: 'Italic (Ctrl+I)',
      active: () => ed.isActive('italic'),
      run: () => ed.chain().focus().toggleItalic().run(),
    }
    map.strike = {
      key: 'strike',
      icon: ICONS.strike,
      title: 'Strikethrough',
      active: () => ed.isActive('strike'),
      run: () => ed.chain().focus().toggleStrike().run(),
    }
    map.superscript = {
      key: 'superscript',
      icon: ICONS.superscript,
      title: 'Superscript',
      active: () => ed.isActive('superscript'),
      run: () => ed.chain().focus().toggleSuperscript().run(),
    }
    map.bulletList = {
      key: 'bulletList',
      icon: ICONS.bulletList,
      title: 'Bullet list',
      active: () => ed.isActive('bulletList'),
      run: () => ed.chain().focus().toggleBulletList().run(),
    }
    map.orderedList = {
      key: 'orderedList',
      icon: ICONS.orderedList,
      title: 'Ordered list',
      active: () => ed.isActive('orderedList'),
      run: () => ed.chain().focus().toggleOrderedList().run(),
    }
    map.blockquote = {
      key: 'blockquote',
      icon: ICONS.blockquote,
      title: 'Quote',
      active: () => ed.isActive('blockquote'),
      run: () => ed.chain().focus().toggleBlockquote().run(),
    }
    map.code = {
      key: 'code',
      icon: ICONS.code,
      title: 'Inline code',
      active: () => ed.isActive('code'),
      run: () => ed.chain().focus().toggleCode().run(),
    }
    map.codeBlock = {
      key: 'codeBlock',
      icon: ICONS.codeBlock,
      title: 'Code block',
      active: () => ed.isActive('codeBlock'),
      run: () => ed.chain().focus().toggleCodeBlock().run(),
    }
  }

  return map
})

function pick(keys: string[]) {
  return keys.map((key) => buttonMap.value[key]).filter((btn): btn is ToolbarButton => !!btn)
}

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
  transition: border-color 0.15s ease;
}

.rte:focus-within {
  border-color: var(--of-text);
}

.rte-bar {
  display: flex;
  align-items: center;
  gap: 1px;
  flex-wrap: wrap;
  padding: 4px 6px;
  border-bottom: 1px solid var(--of-border-soft);
  background: var(--of-bg);
}

.rte-bar-spacer {
  flex: 1;
  min-width: 8px;
}

.rte-sep {
  width: 1px;
  height: 16px;
  margin: 0 4px;
  background: var(--of-border);
}

.rte-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 28px;
  height: 28px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--of-text);
  cursor: pointer;
  transition: background 0.12s ease, color 0.12s ease;
}

.rte-icon {
  display: inline-flex;
  width: 16px;
  height: 16px;
}

.rte-icon :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
}

.rte-btn:hover:not(:disabled) {
  background: var(--of-border-soft);
}

.rte-btn.active {
  background: var(--of-text);
  color: var(--of-bg);
}

.rte-btn:disabled {
  opacity: 0.35;
  cursor: default;
}

.rte-heading-btn {
  font-weight: 600;
}

.rte-count {
  font-size: 12px;
  color: var(--of-text-subtle);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.rte-uploading {
  font-size: 12px;
  color: var(--of-text-muted);
}

.rte-info {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  margin: 0 2px;
  color: var(--of-text-subtle);
  cursor: help;
}

.rte-info .rte-icon {
  width: 14px;
  height: 14px;
}

.rte-switch {
  flex-shrink: 0;
  margin-left: 4px;
  padding: 0 2px;
  border: none;
  background: transparent;
  color: var(--of-text-muted);
  font-size: 12px;
  cursor: pointer;
}

.rte-switch:hover {
  color: var(--of-text);
  text-decoration: underline;
}

.rte-linkbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-bottom: 1px solid var(--of-border-soft);
  background: var(--of-border-soft);
}

.rte-link-input {
  flex: 1;
  min-width: 0;
  height: 28px;
  padding: 0 8px;
  border: 1px solid var(--of-border);
  border-radius: 4px;
  background: var(--of-card);
  color: var(--of-text);
  font-size: 13px;
  outline: none;
}

.rte-link-input:focus {
  border-color: var(--of-text);
}

.rte-link-btn {
  flex-shrink: 0;
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--of-border);
  border-radius: 4px;
  background: var(--of-card);
  color: var(--of-text);
  font-size: 12px;
  cursor: pointer;
}

.rte-link-btn:hover {
  background: var(--of-bg);
  border-color: var(--of-text-muted);
}

.rte-link-btn.primary {
  background: var(--of-text);
  border-color: var(--of-text);
  color: var(--of-bg);
}

.rte-link-btn.primary:hover {
  background: var(--of-primary-dark);
  border-color: var(--of-primary-dark);
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

.rte-editor :deep(.rte-content h1),
.rte-editor :deep(.rte-content h2),
.rte-editor :deep(.rte-content h3) {
  margin: 0 0 10px;
  font-weight: 700;
  line-height: 1.35;
}

.rte-editor :deep(.rte-content h1) {
  font-size: 22px;
}

.rte-editor :deep(.rte-content h2) {
  font-size: 18px;
}

.rte-editor :deep(.rte-content h3) {
  font-size: 16px;
}

.rte-editor :deep(.rte-content blockquote) {
  margin: 0 0 10px;
  padding: 4px 14px;
  border-left: 3px solid var(--of-text);
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
  color: var(--of-text);
  text-decoration: underline;
  text-underline-offset: 2px;
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

.rte-editor :deep(.rte-content video) {
  display: block;
}

.rte-editor :deep(.rte-content table) {
  width: 100%;
  margin: 0 0 10px;
  border-collapse: collapse;
  table-layout: fixed;
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

.rte-editor :deep(.rte-content .selectedCell::after) {
  background: rgba(10, 10, 10, 0.08);
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

.rte-compact .rte-editor :deep(.rte-content) {
  font-size: 14px;
}

.rte-compact .rte-editor-wrap {
  padding: 8px 10px;
}
</style>
