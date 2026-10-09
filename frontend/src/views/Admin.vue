<template>
  <div>
    <h1 class="page-title">管理后台</h1>

    <div v-if="!ready" style="padding: 40px; text-align: center">
      <el-skeleton :rows="6" animated />
    </div>

    <div v-else-if="!isAdmin" class="state-box error">仅管理员可访问</div>

    <el-tabs v-else v-model="activeTab">
      <el-tab-pane label="待审认证" name="auth">
        <div class="admin-filter">
          <span>等级筛选：</span>
          <el-select
            v-model="levelFilter"
            clearable
            placeholder="全部等级"
            size="small"
            style="width: 140px"
            @change="loadAuth"
          >
            <el-option
              v-for="lv in LEVEL_OPTIONS"
              :key="lv.value"
              :label="lv.label"
              :value="lv.value"
            />
          </el-select>
        </div>
        <el-skeleton v-if="authLoading" :rows="4" animated />
        <div v-else-if="!authItems.length" class="state-box">暂无待审认证</div>
        <div v-else class="record-list">
          <div v-for="item in authItems" :key="item.id" class="record-item">
            <div style="flex: 1">
              <div style="font-weight: 600">
                {{ authTypeLabel(item.type) }}
                <el-tag v-if="item.level" size="small" type="info" effect="plain">
                  {{ authLevelLabel(item.level) }}
                </el-tag>
                <span style="color: var(--of-text-muted); font-weight: 400">
                  · 用户 #{{ item.userId ?? item.authorId ?? '-' }}
                </span>
              </div>
              <div class="sidebar-item-meta">
                提交于 {{ formatDate(item.createdAt) }}
              </div>
              <div style="margin-top: 6px; display: flex; flex-wrap: wrap; gap: 6px">
                <el-tag
                  v-for="kv in renderData(item)"
                  :key="kv.key"
                  size="small"
                  effect="plain"
                >
                  {{ kv.key }}: {{ kv.value }}
                </el-tag>
              </div>
              <div v-if="item.media?.length" class="admin-media">
                <template v-for="m in item.media" :key="m.id">
                  <img
                    v-if="m.kind !== 'VIDEO' && mediaUrls[m.id]"
                    :src="mediaUrls[m.id]"
                    class="admin-media-thumb"
                    :alt="`media-${m.id}`"
                    @click="previewMedia(m.id)"
                  />
                  <el-tag v-else-if="m.kind === 'VIDEO'" size="small" type="warning">
                    视频已上传
                  </el-tag>
                  <div v-else class="admin-media-thumb admin-media-loading" />
                </template>
              </div>
              <div v-if="livenessText(item)" style="margin-top: 6px">
                <el-tag size="small" :type="livenessTag(item)">{{ livenessText(item) }}</el-tag>
              </div>
            </div>
            <div style="display: flex; gap: 8px">
              <el-button type="success" size="small" @click="approveAuth(item)">
                通过
              </el-button>
              <el-button type="danger" size="small" plain @click="rejectAuth(item)">
                驳回
              </el-button>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <el-tab-pane label="待审自定义页面" name="pages">
        <el-skeleton v-if="pagesLoading" :rows="4" animated />
        <div v-else-if="!pagesItems.length" class="state-box">暂无待审自定义页面</div>
        <div v-else class="record-list">
          <div v-for="item in pagesItems" :key="item.id" class="record-item">
            <div style="flex: 1">
              <div style="font-weight: 600">
                {{ item.forumName || item.name || item.slug || `页面 #${item.id}` }}
              </div>
              <div class="sidebar-item-meta">
                提交于 {{ formatDate(item.createdAt) }}
              </div>
            </div>
            <div style="display: flex; gap: 8px">
              <el-button size="small" @click="preview(item)">预览</el-button>
              <el-button type="success" size="small" @click="approvePage(item)">
                通过
              </el-button>
              <el-button type="danger" size="small" plain @click="rejectPage(item)">
                驳回
              </el-button>
            </div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>

    <el-dialog v-model="previewVisible" title="自定义页面预览" width="720px">
      <iframe
        v-if="previewHtml"
        class="admin-preview-frame"
        sandbox="allow-popups"
        :srcdoc="previewHtml"
      ></iframe>
    </el-dialog>

    <el-dialog v-model="mediaVisible" title="媒体预览" width="640px">
      <img v-if="mediaPreviewUrl" :src="mediaPreviewUrl" style="max-width: 100%" />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { adminApi, authenticationApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import {
  authLevelLabel,
  authTypeLabel,
  errorMessage,
  formatDate,
  normalizePage,
  safeJsonParse,
} from '@/utils/format'

interface AdminMediaItem {
  id: number
  kind: string
  contentType?: string
}

interface AdminAuthItem {
  id: number
  type: string
  level?: string
  userId?: number
  authorId?: number
  status?: string
  authData?: string
  data?: Record<string, unknown>
  media?: AdminMediaItem[]
  liveness?: Record<string, unknown> | null
  createdAt?: string
}

interface AdminPageItem {
  id: number
  forumId?: number
  forumName?: string
  name?: string
  slug?: string
  html?: string
  customPageHtml?: string
  createdAt?: string
}

const auth = useAuthStore()
const router = useRouter()

const ready = ref(false)
const isAdmin = ref(false)
const activeTab = ref('auth')

const authItems = ref<AdminAuthItem[]>([])
const authLoading = ref(false)

const pagesItems = ref<AdminPageItem[]>([])
const pagesLoading = ref(false)

const previewVisible = ref(false)
const previewHtml = ref('')

const levelFilter = ref<string>('')
const LEVEL_OPTIONS = [
  { value: 'LOW', label: '低' },
  { value: 'MEDIUM', label: '中' },
  { value: 'HIGH', label: '高' },
  { value: 'EXTREME', label: '极高' },
]

const mediaUrls = ref<Record<number, string>>({})
const mediaVisible = ref(false)
const mediaPreviewUrl = ref('')

function livenessText(item: AdminAuthItem): string {
  const l = item.liveness
  if (!l) return ''
  if (l.degraded) return '活体降级（转人工）'
  if (l.passed === true) return '活体已通过'
  return '未通过活体'
}

function livenessTag(item: AdminAuthItem): 'success' | 'warning' | 'danger' {
  const l = item.liveness
  if (!l) return 'warning'
  if (l.degraded) return 'warning'
  return l.passed === true ? 'success' : 'danger'
}

async function loadMediaThumbs(items: AdminAuthItem[]) {
  for (const item of items) {
    for (const m of item.media || []) {
      if (m.kind === 'VIDEO' || mediaUrls.value[m.id]) continue
      try {
        const blob = await authenticationApi.mediaBlob(m.id)
        mediaUrls.value[m.id] = URL.createObjectURL(blob)
      } catch {
        /* 忽略单个媒体加载失败 */
      }
    }
  }
}

function previewMedia(id: number) {
  const url = mediaUrls.value[id]
  if (!url) return
  mediaPreviewUrl.value = url
  mediaVisible.value = true
}

function renderData(item: AdminAuthItem): { key: string; value: string }[] {
  const source =
    item.data && typeof item.data === 'object'
      ? item.data
      : safeJsonParse<Record<string, unknown>>(item.authData || '') || {}
  return Object.entries(source).map(([key, value]) => ({
    key,
    value: typeof value === 'object' ? JSON.stringify(value) : String(value),
  }))
}

async function loadAuth() {
  authLoading.value = true
  try {
    const payload = await adminApi.authentications({
      status: 'PENDING',
      level: levelFilter.value || undefined,
      page: 1,
      size: 50,
    })
    authItems.value = normalizePage<AdminAuthItem>(payload).items
    await loadMediaThumbs(authItems.value)
  } catch (e) {
    ElMessage.error(errorMessage(e, '待审认证加载失败'))
  } finally {
    authLoading.value = false
  }
}

async function loadPages() {
  pagesLoading.value = true
  try {
    const payload = await adminApi.customPages({ status: 'PENDING', page: 1, size: 50 })
    pagesItems.value = normalizePage<AdminPageItem>(payload).items
  } catch (e) {
    ElMessage.error(errorMessage(e, '待审页面加载失败'))
  } finally {
    pagesLoading.value = false
  }
}

async function approveAuth(item: AdminAuthItem) {
  try {
    await adminApi.approveAuthentication(item.id)
    ElMessage.success('已通过')
    loadAuth()
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

async function rejectAuth(item: AdminAuthItem) {
  try {
    const { value } = await ElMessageBox.prompt('请输入驳回原因', '驳回认证', {
      inputPlaceholder: '驳回原因',
      confirmButtonText: '确定',
      cancelButtonText: '取消',
    })
    await adminApi.rejectAuthentication(item.id, value)
    ElMessage.success('已驳回')
    loadAuth()
  } catch {
    /* 取消或失败 */
  }
}

function preview(item: AdminPageItem) {
  previewHtml.value = item.html || item.customPageHtml || ''
  previewVisible.value = true
}

async function approvePage(item: AdminPageItem) {
  try {
    await adminApi.approveCustomPage(item.id)
    ElMessage.success('已通过')
    loadPages()
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

async function rejectPage(item: AdminPageItem) {
  try {
    const { value } = await ElMessageBox.prompt('请输入驳回原因', '驳回自定义页面', {
      inputPlaceholder: '驳回原因',
      confirmButtonText: '确定',
      cancelButtonText: '取消',
    })
    await adminApi.rejectCustomPage(item.id, value)
    ElMessage.success('已驳回')
    loadPages()
  } catch {
    /* 取消或失败 */
  }
}

onMounted(async () => {
  if (auth.isAuthenticated) await auth.fetchMe()
  isAdmin.value = auth.isAdmin
  ready.value = true
  if (!isAdmin.value) {
    ElMessage.error('仅管理员可访问管理后台')
    router.replace('/')
    return
  }
  loadAuth()
  loadPages()
})

onBeforeUnmount(() => {
  Object.values(mediaUrls.value).forEach((url) => URL.revokeObjectURL(url))
  mediaUrls.value = {}
})
</script>

<style scoped>
.admin-filter {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  color: var(--of-text-muted);
  font-size: 13px;
}
.admin-media {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
  align-items: center;
}
.admin-media-thumb {
  width: 72px;
  height: 72px;
  object-fit: cover;
  border-radius: var(--of-radius-sm);
  border: 1px solid var(--of-border);
  cursor: pointer;
  background: var(--of-surface-weak);
}
.admin-media-loading {
  cursor: default;
}
</style>
