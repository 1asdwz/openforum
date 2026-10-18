<template>
  <div class="page-grid no-right">
    <LeftRail />

    <div class="feed-column">
      <div class="content-card editor-head">
        <div>
          <h1 class="page-title" style="margin: 0">主页装修</h1>
          <p class="editor-desc">
            选择普通个人主页，或用 HTML 设计自己的主页。自定义主页保存后立即生效。
          </p>
        </div>
        <el-button round @click="goProfile">查看我的主页</el-button>
      </div>

      <el-alert
        v-if="blocked"
        type="error"
        :closable="false"
        show-icon
        title="你的自定义主页已被管理员下架"
        :description="`原因：${blockReason || '未填写'}。修改内容后仍需管理员复核恢复。`"
      />

      <el-skeleton v-if="loading" :rows="6" animated />

      <template v-else>
        <section class="content-card">
          <h2 class="editor-section-title">选择主页模式</h2>
          <div class="mode-grid">
            <button
              type="button"
              class="mode-card"
              :class="{ active: mode === 'NORMAL' }"
              @click="mode = 'NORMAL'"
            >
              <span class="mode-name">普通个人主页</span>
              <span class="mode-desc">沿用默认的帖子、评论与关于结构。</span>
            </button>
            <button
              type="button"
              class="mode-card"
              :class="{ active: mode === 'CUSTOM' }"
              @click="mode = 'CUSTOM'"
            >
              <span class="mode-name">自定义个人主页</span>
              <span class="mode-desc">上传静态 HTML，用内联样式自由排版。</span>
            </button>
          </div>
        </section>

        <template v-if="mode === 'CUSTOM'">
          <section class="content-card">
            <div class="editor-toolbar">
              <el-button size="small" @click="pickImage">插入图片</el-button>
              <el-button size="small" @click="insertExample">插入示例模板</el-button>
              <el-button size="small" @click="html = ''">清空</el-button>
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
              placeholder="在这里编写 HTML，例如：<div style=&quot;padding:24px&quot;>你好</div>"
            />

            <ul class="editor-tips">
              <li>不支持 JavaScript，<code>&lt;script&gt;</code> 会被移除。</li>
              <li>仅支持站内图片，外链图片会被移除。</li>
              <li>仅支持 <code>style</code> 属性，不支持 <code>&lt;style&gt;</code> 标签。</li>
              <li>外部链接点击时会弹出安全风险提示。</li>
            </ul>
          </section>

          <section class="content-card">
            <h2 class="editor-section-title">实时预览</h2>
            <SafeHomepageFrame :html="html" />
          </section>
        </template>

        <div class="editor-actions">
          <el-button :loading="resetting" @click="resetHomepage">恢复普通主页</el-button>
          <el-button
            type="primary"
            round
            :loading="saving"
            :disabled="mode === 'CUSTOM' && !canSave"
            @click="save"
          >
            保存
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

const MAX_BYTES = 65_536

const EXAMPLE_HTML = `<section style="padding:32px;border-radius:16px;background:#f6f8ff">
  <h1 style="margin:0 0 12px;font-size:28px">你好，我是……</h1>
  <p style="margin:0 0 16px;line-height:1.8;color:#4b5563">
    欢迎来到我的主页。这里可以写下自我介绍、兴趣方向与联系方式。
  </p>
  <p style="margin:0">
    <a href="https://example.com" style="color:#2563eb">我的博客</a>
  </p>
</section>`

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
    ElMessage.error(errorMessage(e, '主页配置加载失败'))
  } finally {
    loading.value = false
  }
}

async function save() {
  if (mode.value === 'CUSTOM' && !canSave.value) {
    ElMessage.warning('请检查内容大小')
    return
  }
  saving.value = true
  try {
    const data = await homepageApi.save({
      mode: mode.value,
      html: mode.value === 'CUSTOM' ? html.value : undefined,
    })
    applyState(data)
    ElMessage.success('已保存')
  } catch (e) {
    ElMessage.error(errorMessage(e, '保存失败'))
  } finally {
    saving.value = false
  }
}

async function resetHomepage() {
  try {
    await ElMessageBox.confirm(
      '将删除自定义内容与全部举报记录，并恢复为普通个人主页。是否继续？',
      '恢复普通主页',
      { confirmButtonText: '确认恢复', cancelButtonText: '取消', type: 'warning' }
    )
  } catch {
    return
  }
  resetting.value = true
  try {
    applyState(await homepageApi.reset())
    ElMessage.success('已恢复普通主页')
  } catch (e) {
    ElMessage.error(errorMessage(e, '操作失败'))
  } finally {
    resetting.value = false
  }
}

function pickImage() {
  fileRef.value?.click()
}

async function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  if (!file.type.startsWith('image/')) {
    ElMessage.warning('请选择图片文件')
    return
  }
  if (file.size > 10 * 1024 * 1024) {
    ElMessage.warning('图片不能超过 10MB')
    return
  }
  uploading.value = true
  try {
    const media = await mediaApi.upload(file)
    insertAtCursor(`<img src="/forum/api/media/${media.id}" alt="">`)
    ElMessage.success('图片已插入')
  } catch (e) {
    ElMessage.error(errorMessage(e, '图片上传失败'))
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

onBeforeRouteLeave(async () => {
  if (!dirty.value) return true
  try {
    await ElMessageBox.confirm('有未保存的修改，确定离开吗？', '离开页面', {
      confirmButtonText: '离开',
      cancelButtonText: '继续编辑',
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
