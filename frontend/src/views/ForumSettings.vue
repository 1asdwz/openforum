<template>
  <div>
    <el-skeleton v-if="loading" :rows="8" animated />

    <div v-else-if="error" class="state-box error">
      {{ error }}
      <div style="margin-top: 12px">
        <el-button size="small" @click="loadForum">Retry</el-button>
      </div>
    </div>

    <div v-else-if="forum && !canManage" class="state-box error">
      You need "Config" permission to change community settings.
    </div>

    <template v-else-if="forum">
      <div style="margin-bottom: 12px">
        <el-button size="small" @click="router.push(`/f/${forum.slug}/mod`)">
          Back to moderation console
        </el-button>
      </div>
      <h1 class="page-title">Manage {{ forum.name }}</h1>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="Basic info" name="basic">
          <el-form label-position="top">
            <el-form-item label="Name">
              <el-input v-model="form.name" />
            </el-form-item>
            <el-form-item label="Description">
              <el-input v-model="form.description" type="textarea" :rows="3" />
            </el-form-item>
            <el-form-item label="Icon (image URL)">
              <el-input v-model="form.icon" />
            </el-form-item>
            <el-form-item label="Theme color">
              <el-color-picker v-model="form.themeColor" />
            </el-form-item>
            <el-form-item label="Browse threshold">
              <GateRuleEditor v-model="form.browseRule" :fields="flatFields" />
            </el-form-item>
            <el-form-item label="Action threshold (post / comment / vote)">
              <GateRuleEditor v-model="form.operateRule" :fields="flatFields" />
            </el-form-item>
            <el-form-item label="Community rules">
              <el-input v-model="form.rules" type="textarea" :rows="4" />
            </el-form-item>
            <div class="form-actions">
              <el-button type="primary" :loading="saving" @click="saveBasic">
                Save changes
              </el-button>
            </div>
          </el-form>
        </el-tab-pane>

        <el-tab-pane label="Default template" name="template">
          <el-form label-position="top">
            <el-form-item label="Announcement">
              <el-input
                v-model="form.announcement"
                type="textarea"
                :rows="3"
                placeholder="Announcement shown at the top of the forum page"
              />
            </el-form-item>

            <el-form-item label="Sections">
              <div v-for="(s, i) in form.sections" :key="i" class="template-row">
                <el-input v-model="s.title" placeholder="Title" style="width: 200px" />
                <el-input v-model="s.content" placeholder="Content" />
                <el-button text type="danger" @click="removeSection(i)">Delete</el-button>
              </div>
              <el-button size="small" @click="addSection">+ Add section</el-button>
            </el-form-item>

            <el-form-item label="Related links">
              <div v-for="(l, i) in form.links" :key="i" class="template-row">
                <el-input v-model="l.label" placeholder="Label" style="width: 180px" />
                <el-input v-model="l.url" placeholder="https://…" />
                <el-button text type="danger" @click="removeLink(i)">Delete</el-button>
              </div>
              <el-button size="small" @click="addLink">+ Add link</el-button>
            </el-form-item>

            <div class="form-actions">
              <el-button type="primary" :loading="saving" @click="saveBasic">
                Save changes
              </el-button>
            </div>
          </el-form>
        </el-tab-pane>

        <el-tab-pane label="Custom homepage" name="homepage">
          <el-alert
            v-if="homepageBlocked"
            type="warning"
            show-icon
            :closable="false"
            title="This custom homepage has been taken down by an administrator, so visitors currently see the default homepage. Changes will not restore it automatically; contact an administrator for review."
            style="margin-bottom: 12px"
          >
            <template #default>Reason: {{ homepageBlockReason || 'Not provided' }}</template>
          </el-alert>

          <el-form label-position="top">
            <el-form-item label="Homepage mode">
              <el-radio-group v-model="homepageMode">
                <el-radio value="NORMAL">Default homepage</el-radio>
                <el-radio value="CUSTOM">Custom homepage</el-radio>
              </el-radio-group>
            </el-form-item>
          </el-form>

          <p class="homepage-hint">
            The custom homepage replaces the main content area on the left of the forum page while
            keeping site navigation and the right sidebar. HTML is sanitized against an allowlist
            before saving and rendered in a sandboxed iframe; read-only data components are
            supported.
          </p>

          <template v-if="homepageMode === 'CUSTOM'">
            <div class="homepage-toolbar">
              <el-button size="small" :loading="uploading" @click="pickImage">
                Insert image
              </el-button>
              <el-button size="small" @click="insertExample">Insert full example</el-button>
              <el-button size="small" @click="goGuide">Component docs</el-button>
              <el-button size="small" @click="homepageHtml = ''">Clear</el-button>
              <span class="byte-count" :class="{ over: homepageBytes > MAX_BYTES }">
                {{ homepageHtml.length }} characters · {{ sizeText }} / 64 KB
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
              v-model="homepageHtml"
              class="html-editor"
              spellcheck="false"
              placeholder="Write HTML here, for example: <div style=&quot;padding:24px&quot;>Welcome to the community</div>"
            />
            <ul class="homepage-tips">
              <li>JavaScript and <code>&lt;script&gt;</code> are not supported; event attributes are removed.</li>
              <li>Only the <code>style</code> attribute is supported; the <code>&lt;style&gt;</code> tag is not.</li>
              <li>Images must be uploaded to the site; externally linked images are removed.</li>
              <li>Clicking external links shows a security warning.</li>
              <li>
                Use <code>&lt;of-forum&gt;</code>, <code>&lt;of-forum-stats&gt;</code>,
                <code>&lt;of-forum-posts&gt;</code>, and <code>&lt;of-members&gt;</code>
                to display forum details, stats, posts, and members.
              </li>
            </ul>
          </template>

          <div class="form-actions">
            <el-button :loading="resettingHomepage" @click="resetHomepage">
              Reset to default homepage
            </el-button>
            <el-button
              type="primary"
              :loading="savingHomepage"
              :disabled="homepageMode === 'CUSTOM' && !canSaveHomepage"
              @click="saveHomepage"
            >
              Save
            </el-button>
            <el-button text @click="goGuide">Open customization guide</el-button>
          </div>
        </el-tab-pane>
      </el-tabs>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import GateRuleEditor from '@/components/GateRuleEditor.vue'
import { forumApi, infoApi, mediaApi } from '@/api'
import type {
  ForumHomepageMode,
  GateRule,
  HomepageStatus,
  InfoField,
  InfoFieldGroup,
  MyForumHomepage,
  SubForum,
} from '@/types'
import { errorMessage, parseTemplateConfig } from '@/utils/format'
import { FORUM_HOMEPAGE_EXAMPLE_HTML } from '@/utils/forumHomepageGuide'

const MAX_BYTES = 65_536

const route = useRoute()
const router = useRouter()

const fieldGroups = ref<InfoFieldGroup[]>([])
const flatFields = computed<InfoField[]>(() =>
  fieldGroups.value.flatMap((g) => g.fields || [])
)

const forum = ref<SubForum | null>(null)
const loading = ref(true)
const error = ref('')
const activeTab = ref('basic')
const saving = ref(false)

const homepageMode = ref<ForumHomepageMode>('NORMAL')
const homepageHtml = ref('')
const homepageStatus = ref<HomepageStatus>('ACTIVE')
const homepageBlockReason = ref<string | null>(null)
const savingHomepage = ref(false)
const resettingHomepage = ref(false)
const uploading = ref(false)
const fileRef = ref<HTMLInputElement | null>(null)
const editorRef = ref<HTMLTextAreaElement | null>(null)

const form = reactive({
  name: '',
  description: '',
  icon: '',
  themeColor: '#FF4500',
  templateType: 'DEFAULT',
  browseRule: null as GateRule | null,
  operateRule: null as GateRule | null,
  rules: '',
  announcement: '',
  sections: [] as { title: string; content: string }[],
  links: [] as { label: string; url: string }[],
})

const canManage = computed(() => {
  const permissions = forum.value?.myPermissions || []
  return (
    !!forum.value?.isCreator ||
    permissions.includes('EVERYTHING') ||
    permissions.includes('CONFIG')
  )
})
const homepageBlocked = computed(
  () => homepageStatus.value === 'BLOCKED' && homepageMode.value === 'CUSTOM'
)
const homepageBytes = computed(() => new TextEncoder().encode(homepageHtml.value).length)
const sizeText = computed(() => {
  const bytes = homepageBytes.value
  return bytes >= 1024 ? `${(bytes / 1024).toFixed(1)} KB` : `${bytes} B`
})
const canSaveHomepage = computed(
  () => homepageBytes.value > 0 && homepageBytes.value <= MAX_BYTES && !uploading.value
)

async function loadForum() {
  loading.value = true
  error.value = ''
  try {
    const data = await forumApi.get(String(route.params.slug))
    forum.value = data
    form.name = data.name || ''
    form.description = data.description || ''
    form.icon = data.icon || ''
    form.themeColor = data.themeColor || '#FF4500'
    form.templateType = data.templateType || 'DEFAULT'
    form.rules = data.rules || ''
    form.browseRule = data.browseRule || null
    form.operateRule = data.operateRule || null

    const tpl = parseTemplateConfig(data.templateConfig)
    form.announcement = tpl.announcement || ''
    form.sections = (tpl.sections || []).map((s) => ({
      title: s.title || '',
      content: s.content || '',
    }))
    form.links = (tpl.links || []).map((l) => ({
      label: l.label || '',
      url: l.url || '',
    }))

    if (canManage.value && data.id) {
      await loadHomepage(data.id)
    }
  } catch (e) {
    error.value = errorMessage(e, 'Failed to load forum')
  } finally {
    loading.value = false
  }
}

function applyHomepage(data: MyForumHomepage) {
  homepageMode.value = data.mode === 'CUSTOM' ? 'CUSTOM' : 'NORMAL'
  homepageHtml.value = data.html || ''
  homepageStatus.value = data.status === 'BLOCKED' ? 'BLOCKED' : 'ACTIVE'
  homepageBlockReason.value = data.blockReason
}

async function loadHomepage(id: number) {
  try {
    applyHomepage(await forumApi.myHomepage(id))
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to load homepage settings'))
  }
}

async function saveBasic() {
  if (!forum.value) return
  saving.value = true
  try {
    await forumApi.update(forum.value.id, {
      name: form.name.trim(),
      description: form.description.trim(),
      icon: form.icon.trim(),
      themeColor: form.themeColor,
      templateType: form.templateType,
      templateConfig: JSON.stringify({
        announcement: form.announcement.trim(),
        sections: form.sections.filter((s) => s.title.trim() || s.content.trim()),
        links: form.links.filter((l) => l.label.trim() && l.url.trim()),
      }),
      rules: form.rules.trim(),
      browseRule: form.browseRule,
      operateRule: form.operateRule,
    })
    ElMessage.success('Saved')
    await loadForum()
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to save'))
  } finally {
    saving.value = false
  }
}

function addSection() {
  form.sections.push({ title: '', content: '' })
}

function removeSection(index: number) {
  form.sections.splice(index, 1)
}

function addLink() {
  form.links.push({ label: '', url: '' })
}

function removeLink(index: number) {
  form.links.splice(index, 1)
}

async function saveHomepage() {
  if (!forum.value) return
  if (homepageMode.value === 'CUSTOM' && !canSaveHomepage.value) {
    ElMessage.warning('Please check the content size')
    return
  }
  savingHomepage.value = true
  try {
    const data = await forumApi.saveHomepage(forum.value.id, {
      mode: homepageMode.value,
      html: homepageMode.value === 'CUSTOM' ? homepageHtml.value : undefined,
    })
    applyHomepage(data)
    ElMessage.success('Saved')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to save'))
  } finally {
    savingHomepage.value = false
  }
}

async function resetHomepage() {
  if (!forum.value) return
  try {
    await ElMessageBox.confirm(
      'This will delete the custom homepage and all report records, and restore the default homepage. Continue?',
      'Reset to default homepage',
      { confirmButtonText: 'Reset', cancelButtonText: 'Cancel', type: 'warning' }
    )
  } catch {
    return
  }
  resettingHomepage.value = true
  try {
    applyHomepage(await forumApi.resetHomepage(forum.value.id))
    ElMessage.success('Reset to default homepage')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Operation failed'))
  } finally {
    resettingHomepage.value = false
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
    ElMessage.warning('Please select an image file')
    return
  }
  if (file.size > 10 * 1024 * 1024) {
    ElMessage.warning('Image must not exceed 10MB')
    return
  }
  uploading.value = true
  try {
    const media = await mediaApi.upload(file)
    insertAtCursor(`<img src="/forum/api/media/${media.id}" alt="">`)
    ElMessage.success('Image inserted')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to upload image'))
  } finally {
    uploading.value = false
  }
}

function insertAtCursor(text: string) {
  const el = editorRef.value
  if (!el) {
    homepageHtml.value += text
    return
  }
  const start = el.selectionStart ?? homepageHtml.value.length
  const end = el.selectionEnd ?? start
  homepageHtml.value =
    homepageHtml.value.slice(0, start) + text + homepageHtml.value.slice(end)
  requestAnimationFrame(() => {
    el.focus()
    const pos = start + text.length
    el.setSelectionRange(pos, pos)
  })
}

function insertExample() {
  insertAtCursor(FORUM_HOMEPAGE_EXAMPLE_HTML)
}

function goGuide() {
  router.push('/settings/forum-homepage/guide')
}

onMounted(async () => {
  loadForum()
  try {
    fieldGroups.value = (await infoApi.fields()) || []
  } catch {
    // If loading fields fails, the threshold editor stays empty; other settings are unaffected
  }
})
</script>

<style scoped>
.homepage-hint {
  margin: 0 0 12px;
  color: var(--of-text-muted);
  font-size: 13px;
  line-height: 1.8;
}
.homepage-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}
.byte-count {
  margin-left: auto;
  color: var(--of-text-muted);
  font-size: 12px;
}
.byte-count.over {
  color: #d03050;
}
.html-editor {
  width: 100%;
  min-height: 360px;
  padding: 12px 14px;
  border: 1px solid var(--of-border);
  border-radius: var(--of-radius);
  background: var(--of-surface-weak);
  color: var(--of-text);
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 13px;
  line-height: 1.7;
  resize: vertical;
  box-sizing: border-box;
}
.html-editor:focus {
  outline: none;
  border-color: var(--of-brand);
}
.homepage-tips {
  margin: 12px 0 0;
  padding-left: 18px;
  color: var(--of-text-muted);
  font-size: 12px;
  line-height: 1.9;
}
.template-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
}
</style>
