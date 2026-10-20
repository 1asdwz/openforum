<template>
  <div class="page-grid no-right">
    <LeftRail />

    <div class="feed-column">
      <div v-reveal class="content-card editor-head">
        <div>
          <span class="of-eyebrow">Homepage</span>
          <h1 class="page-title" style="margin: 0">Homepage editor</h1>
          <p class="editor-desc">
            Choose the standard profile homepage, or design your own with HTML. A custom homepage takes effect immediately after saving.
          </p>
        </div>
        <el-button round @click="goProfile">View my homepage</el-button>
      </div>

      <el-alert
        v-if="blocked"
        type="error"
        :closable="false"
        show-icon
        title="Your custom homepage was taken down by an admin"
        :description="`Reason: ${blockReason || 'Not provided'}. Even after editing, an admin must review it to restore it.`"
      />

      <el-skeleton v-if="loading" :rows="6" animated />

      <template v-else>
        <section class="content-card">
          <h2 class="editor-section-title">Choose homepage mode</h2>
          <div class="mode-grid">
            <button
              type="button"
              class="mode-card"
              :class="{ active: mode === 'NORMAL' }"
              @click="mode = 'NORMAL'"
            >
              <span class="mode-name">Standard profile homepage</span>
              <span class="mode-desc">Uses the default posts, comments, and about layout.</span>
            </button>
            <button
              type="button"
              class="mode-card"
              :class="{ active: mode === 'CUSTOM' }"
              @click="mode = 'CUSTOM'"
            >
              <span class="mode-name">Custom profile homepage</span>
              <span class="mode-desc">Upload static HTML and lay it out freely with inline styles.</span>
            </button>
          </div>
        </section>

        <template v-if="mode === 'CUSTOM'">
          <section class="content-card">
            <div class="editor-toolbar">
              <el-button size="small" @click="pickImage">Insert image</el-button>
              <el-button size="small" @click="insertExample">Insert full example</el-button>
              <el-button size="small" @click="goGuide">Component docs</el-button>
              <el-button size="small" @click="html = ''">Clear</el-button>
              <span class="byte-count" :class="{ over: byteCount > MAX_BYTES }">
                {{ sizeText }} / 64 KB
              </span>
            </div>
            <input
              ref="fileRef"
              type="file"
              accept="image/jpeg,image/png,image/webp,image/gif"
              style="display: none"
              @change="onFileChange"
            />
            <textarea
              ref="editorRef"
              v-model="html"
              class="html-editor"
              spellcheck="false"
              placeholder="Write HTML here, e.g. <div style=&quot;padding:24px&quot;>Hello</div>"
            />

            <ul class="editor-tips">
              <li>JavaScript is not supported; <code>&lt;script&gt;</code> is removed.</li>
              <li>Only in-site images are supported; external images are removed.</li>
              <li>Only the <code>style</code> attribute is supported; <code>&lt;style&gt;</code> tags are not.</li>
              <li>Clicking an external link shows a security warning.</li>
              <li>
                Use <code>&lt;of-profile&gt;</code>, <code>&lt;of-stats&gt;</code>, and <code>&lt;of-posts&gt;</code>
                to show public info, stats, and post lists. See
                <el-button link type="primary" size="small" @click="goGuide">Component docs</el-button>.
              </li>
            </ul>
          </section>

          <section class="content-card">
            <h2 class="editor-section-title">Live preview</h2>
            <SafeHomepageFrame :html="html" :load-data="loadPreviewData" />
          </section>
        </template>

        <div class="editor-actions">
          <el-button :loading="resetting" @click="resetHomepage">Restore standard homepage</el-button>
          <el-button
            type="primary"
            round
            :loading="saving"
            :disabled="mode === 'CUSTOM' && !canSave"
            @click="save"
          >
            Save
          </el-button>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { onBeforeRouteLeave, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import LeftRail from '@/components/LeftRail.vue'
import SafeHomepageFrame from '@/components/SafeHomepageFrame.vue'
import { homepageApi, mediaApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { HomepageMode, MyHomepage } from '@/types'
import { errorMessage } from '@/utils/format'
import { HOMEPAGE_EXAMPLE_HTML } from '@/utils/homepageGuide'

const MAX_BYTES = 65_536

const EXAMPLE_HTML = HOMEPAGE_EXAMPLE_HTML

const router = useRouter()
const auth = useAuthStore()

const loading = ref(true)
const saving = ref(false)
const resetting = ref(false)
const uploading = ref(false)

const mode = ref<HomepageMode>('NORMAL')
const html = ref('')
const status = ref<'ACTIVE' | 'BLOCKED'>('ACTIVE')
const blockReason = ref<string | null>(null)
const savedMode = ref<HomepageMode>('NORMAL')
const savedHtml = ref('')

const fileRef = ref<HTMLInputElement | null>(null)
const editorRef = ref<HTMLTextAreaElement | null>(null)

const blocked = computed(() => status.value === 'BLOCKED' && mode.value === 'CUSTOM')

const byteCount = computed(() => new TextEncoder().encode(html.value).length)
const sizeText = computed(() => {
  const bytes = byteCount.value
  return bytes >= 1024 ? `${(bytes / 1024).toFixed(1)} KB` : `${bytes} B`
})
const canSave = computed(
  () => byteCount.value > 0 && byteCount.value <= MAX_BYTES && !uploading.value
)

const dirty = computed(
  () => mode.value !== savedMode.value || (mode.value === 'CUSTOM' && html.value !== savedHtml.value)
)

function applyState(data: MyHomepage) {
  mode.value = data.mode === 'CUSTOM' ? 'CUSTOM' : 'NORMAL'
  html.value = data.html || ''
  status.value = data.status === 'BLOCKED' ? 'BLOCKED' : 'ACTIVE'
  blockReason.value = data.blockReason
  savedMode.value = mode.value
  savedHtml.value = html.value
}

async function load() {
  loading.value = true
  try {
    applyState(await homepageApi.mine())
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to load homepage settings'))
  } finally {
    loading.value = false
  }
}

async function save() {
  if (mode.value === 'CUSTOM' && !canSave.value) {
    ElMessage.warning('Check the content size')
    return
  }
  saving.value = true
  try {
    const data = await homepageApi.save({
      mode: mode.value,
      html: mode.value === 'CUSTOM' ? html.value : undefined,
    })
    applyState(data)
    ElMessage.success('Saved')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Save failed'))
  } finally {
    saving.value = false
  }
}

async function resetHomepage() {
  try {
    await ElMessageBox.confirm(
      'This deletes the custom content and all report records and restores the standard profile homepage. Continue?',
      'Restore standard homepage',
      { confirmButtonText: 'Restore', cancelButtonText: 'Cancel', type: 'warning' }
    )
  } catch {
    return
  }
  resetting.value = true
  try {
    applyState(await homepageApi.reset())
    ElMessage.success('Standard homepage restored')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Operation failed'))
  } finally {
    resetting.value = false
  }
}

function pickImage() {
  fileRef.value?.click()
}

/** The live preview loads data components through a separate request, matching the public homepage. */
function loadPreviewData() {
  const id = auth.user?.id
  if (!id) return Promise.reject(new Error('Not signed in'))
  return homepageApi.data(id)
}

async function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  if (!file.type.startsWith('image/')) {
    ElMessage.warning('Please select an image file')
    return
  }
  if (file.size > 10 * 1024 * 1024) {
    ElMessage.warning('Image must be 10 MB or smaller')
    return
  }
  uploading.value = true
  try {
    const media = await mediaApi.upload(file)
    insertAtCursor(`<img src="/forum/api/media/${media.id}" alt="">`)
    ElMessage.success('Image inserted')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Image upload failed'))
  } finally {
    uploading.value = false
  }
}

function insertAtCursor(text: string) {
  const el = editorRef.value
  if (!el) {
    html.value += text
    return
  }
  const start = el.selectionStart ?? html.value.length
  const end = el.selectionEnd ?? start
  html.value = html.value.slice(0, start) + text + html.value.slice(end)
  requestAnimationFrame(() => {
    el.focus()
    const pos = start + text.length
    el.setSelectionRange(pos, pos)
  })
}

function insertExample() {
  insertAtCursor(EXAMPLE_HTML)
}

function goProfile() {
  if (auth.user) router.push(`/u/${auth.user.id}`)
}

function goGuide() {
  router.push('/settings/homepage/guide')
}

onBeforeRouteLeave(async () => {
  if (!dirty.value) return true
  try {
    await ElMessageBox.confirm('You have unsaved changes. Leave anyway?', 'Leave page', {
      confirmButtonText: 'Leave',
      cancelButtonText: 'Keep editing',
      type: 'warning',
    })
    return true
  } catch {
    return false
  }
})

onMounted(load)
</script>

<style scoped>
.editor-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}
.editor-desc {
  margin: 8px 0 0;
  color: var(--of-text-muted);
  font-size: 13px;
}
.editor-section-title {
  margin: 0 0 16px;
  font-size: 16px;
}
.mode-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}
.mode-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 18px;
  text-align: left;
  border: 1px solid var(--of-border);
  border-radius: var(--of-radius);
  background: var(--of-surface);
  color: inherit;
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease;
}
.mode-card.active {
  border-color: var(--of-brand);
  background: var(--of-surface-weak);
}
.mode-name {
  font-weight: 600;
}
.mode-desc {
  font-size: 13px;
  color: var(--of-text-muted);
  line-height: 1.6;
}
.editor-toolbar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}
.byte-count {
  margin-left: auto;
  font-size: 12px;
  color: var(--of-text-muted);
}
.byte-count.over {
  color: var(--of-danger, #d93025);
  font-weight: 600;
}
.html-editor {
  width: 100%;
  min-height: 320px;
  padding: 14px;
  border: 1px solid var(--of-border);
  border-radius: var(--of-radius);
  background: var(--of-surface-weak);
  color: var(--of-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 13px;
  line-height: 1.7;
  resize: vertical;
}
.html-editor:focus {
  outline: none;
  border-color: var(--of-brand);
}
.editor-tips {
  margin: 12px 0 0;
  padding-left: 18px;
  color: var(--of-text-muted);
  font-size: 12px;
  line-height: 1.9;
}
.editor-tips code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}
.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
@media (max-width: 720px) {
  .mode-grid {
    grid-template-columns: 1fr;
  }
  .editor-head {
    flex-direction: column;
  }
}
</style>
