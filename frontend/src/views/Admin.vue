<template>
  <div>
    <header v-reveal class="page-head">
      <span class="of-eyebrow">Operations</span>
      <h1 class="page-title">Admin Console</h1>
      <p class="page-desc">
        Review verification requests and custom community homepages before they go live.
      </p>
    </header>

    <div v-if="!ready" style="padding: 40px; text-align: center">
      <el-skeleton :rows="6" animated />
    </div>

    <div v-else-if="!isAdmin" class="state-box error">Admins only</div>

    <el-tabs v-else v-model="activeTab">
      <el-tab-pane label="Pending verifications" name="auth">
        <div class="admin-filter">
          <span>Level filter:</span>
          <el-select
            v-model="levelFilter"
            clearable
            placeholder="All levels"
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
        <div v-else-if="!authItems.length" class="state-box">No pending verifications</div>
        <div v-else class="record-list">
          <div v-for="item in authItems" :key="item.id" class="record-item">
            <div style="flex: 1">
              <div style="font-weight: 600">
                {{ authTypeLabel(item.type) }}
                <el-tag v-if="item.level" size="small" type="info" effect="plain">
                  {{ authLevelLabel(item.level) }}
                </el-tag>
                <span style="color: var(--of-text-muted); font-weight: 400">
                  · User #{{ item.userId ?? item.authorId ?? '-' }}
                </span>
              </div>
              <div class="sidebar-item-meta">
                Submitted {{ formatDate(item.createdAt) }}
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
                    Video uploaded
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
                Approve
              </el-button>
              <el-button type="danger" size="small" plain @click="rejectAuth(item)">
                Reject
              </el-button>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <el-tab-pane label="Forum homepages" name="forum-homepages">
        <div class="admin-filter">
          <span>Status filter:</span>
          <el-select
            v-model="forumHomepageFilter"
            size="small"
            style="width: 140px"
            @change="loadForumHomepages"
          >
            <el-option label="All" value="ALL" />
            <el-option label="Published" value="ACTIVE" />
            <el-option label="Blocked" value="BLOCKED" />
          </el-select>
        </div>
        <el-skeleton v-if="forumHomepagesLoading" :rows="4" animated />
        <div v-else-if="!forumHomepageItems.length" class="state-box">No custom forum homepages</div>
        <div v-else class="record-list">
          <div
            v-for="item in forumHomepageItems"
            :key="item.forumId"
            class="record-item"
          >
            <div style="flex: 1">
              <div style="font-weight: 600">
                {{ item.forumName }}
                <span class="sidebar-item-meta">f/{{ item.forumSlug }}</span>
                <el-tag
                  size="small"
                  :type="item.status === 'BLOCKED' ? 'danger' : 'success'"
                  effect="plain"
                >
                  {{ item.status === 'BLOCKED' ? 'Blocked' : 'Published' }}
                </el-tag>
              </div>
              <div class="sidebar-item-meta">
                Owner {{ item.creatorName }} · Size {{ sizeText(item.sizeBytes) }} · Reports
                {{ item.reportCount }} · Updated {{ formatDate(item.updatedAt) || '-' }}
              </div>
              <div v-if="item.blockReason" class="sidebar-item-meta">
                Block reason: {{ item.blockReason }}
              </div>
            </div>
            <div style="display: flex; gap: 8px">
              <el-button size="small" @click="openForum(item)">View</el-button>
              <el-button
                v-if="item.status === 'BLOCKED'"
                type="success"
                size="small"
                @click="unblockForumHomepage(item)"
              >
                Restore
              </el-button>
              <el-button
                v-else
                type="danger"
                size="small"
                plain
                @click="blockForumHomepage(item)"
              >
                Block
              </el-button>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <el-tab-pane label="Profile pages" name="homepages">
        <div class="admin-filter">
          <span>Status filter:</span>
          <el-select
            v-model="homepageFilter"
            size="small"
            style="width: 140px"
            @change="loadHomepages"
          >
            <el-option label="All" value="ALL" />
            <el-option label="Published" value="ACTIVE" />
            <el-option label="Blocked" value="BLOCKED" />
          </el-select>
        </div>
        <el-skeleton v-if="homepagesLoading" :rows="4" animated />
        <div v-else-if="!homepageItems.length" class="state-box">No custom profile pages</div>
        <div v-else class="record-list">
          <div v-for="item in homepageItems" :key="item.userId" class="record-item">
            <div style="flex: 1">
              <div style="font-weight: 600">
                {{ item.username }}
                <el-tag
                  size="small"
                  :type="item.status === 'BLOCKED' ? 'danger' : 'success'"
                  effect="plain"
                >
                  {{ item.status === 'BLOCKED' ? 'Blocked' : 'Published' }}
                </el-tag>
              </div>
              <div class="sidebar-item-meta">
                Size {{ sizeText(item.sizeBytes) }} · Reports {{ item.reportCount }} · Updated
                {{ formatDate(item.updatedAt) || '-' }}
              </div>
              <div v-if="item.blockReason" class="sidebar-item-meta">
                Block reason: {{ item.blockReason }}
              </div>
            </div>
            <div style="display: flex; gap: 8px">
              <el-button size="small" @click="openHomepage(item)">View</el-button>
              <el-button
                v-if="item.status === 'BLOCKED'"
                type="success"
                size="small"
                @click="unblockHomepage(item)"
              >
                Restore
              </el-button>
              <el-button
                v-else
                type="danger"
                size="small"
                plain
                @click="blockHomepage(item)"
              >
                Block
              </el-button>
            </div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>

    <el-dialog v-model="mediaVisible" title="Media preview" width="640px">
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
import type { AdminForumHomepageItem, AdminHomepageItem } from '@/types'
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

const auth = useAuthStore()
const router = useRouter()

const ready = ref(false)
const isAdmin = ref(false)
const activeTab = ref('auth')

const authItems = ref<AdminAuthItem[]>([])
const authLoading = ref(false)

const levelFilter = ref<string>('')
const LEVEL_OPTIONS = [
  { value: 'LOW', label: 'Low' },
  { value: 'MEDIUM', label: 'Medium' },
  { value: 'HIGH', label: 'High' },
  { value: 'EXTREME', label: 'Extreme' },
]

const mediaUrls = ref<Record<number, string>>({})
const mediaVisible = ref(false)
const mediaPreviewUrl = ref('')

const homepageItems = ref<AdminHomepageItem[]>([])
const homepagesLoading = ref(false)
const homepageFilter = ref('ALL')

function sizeText(bytes: number): string {
  if (!bytes) return '0 B'
  return bytes >= 1024 ? `${(bytes / 1024).toFixed(1)} KB` : `${bytes} B`
}

async function loadHomepages() {
  homepagesLoading.value = true
  try {
    const payload = await adminApi.userHomepages({
      status: homepageFilter.value,
      page: 1,
      size: 50,
    })
    homepageItems.value = normalizePage<AdminHomepageItem>(payload).items
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to load profile pages'))
  } finally {
    homepagesLoading.value = false
  }
}

function openHomepage(item: AdminHomepageItem) {
  window.open(`${import.meta.env.BASE_URL}u/${item.userId}`, '_blank', 'noopener,noreferrer')
}

async function blockHomepage(item: AdminHomepageItem) {
  try {
    const { value } = await ElMessageBox.prompt(
      `Block ${item.username}'s custom profile page. Please provide a reason`,
      'Block Profile Page',
      {
        inputPlaceholder: 'Block reason (2-200 characters)',
        confirmButtonText: 'Block',
        cancelButtonText: 'Cancel',
        inputValidator: (v: string) => {
          const len = (v || '').trim().length
          return len >= 2 && len <= 200 ? true : 'Reason must be 2-200 characters'
        },
      }
    )
    await adminApi.blockUserHomepage(item.userId, value.trim())
    ElMessage.success('Blocked')
    loadHomepages()
  } catch {
    /* cancelled or failed */
  }
}

async function unblockHomepage(item: AdminHomepageItem) {
  try {
    await ElMessageBox.confirm(
      `Restore ${item.username}'s custom profile page? It will be public again immediately.`,
      'Restore Profile Page',
      { confirmButtonText: 'Restore', cancelButtonText: 'Cancel', type: 'warning' }
    )
  } catch {
    return
  }
  try {
    await adminApi.unblockUserHomepage(item.userId)
    ElMessage.success('Restored')
    loadHomepages()
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to restore'))
  }
}

function livenessText(item: AdminAuthItem): string {
  const l = item.liveness
  if (!l) return ''
  if (l.degraded) return 'Liveness degraded (manual review)'
  if (l.passed === true) return 'Liveness passed'
  return 'Liveness failed'
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
        /* ignore individual media load failures */
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
    ElMessage.error(errorMessage(e, 'Failed to load pending verifications'))
  } finally {
    authLoading.value = false
  }
}

const forumHomepageItems = ref<AdminForumHomepageItem[]>([])
const forumHomepagesLoading = ref(false)
const forumHomepageFilter = ref('ALL')

async function loadForumHomepages() {
  forumHomepagesLoading.value = true
  try {
    const payload = await adminApi.forumHomepages({
      status: forumHomepageFilter.value,
      page: 1,
      size: 50,
    })
    forumHomepageItems.value = normalizePage<AdminForumHomepageItem>(payload).items
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to load forum homepages'))
  } finally {
    forumHomepagesLoading.value = false
  }
}

function openForum(item: AdminForumHomepageItem) {
  window.open(`${import.meta.env.BASE_URL}f/${item.forumSlug}`, '_blank', 'noopener,noreferrer')
}

async function blockForumHomepage(item: AdminForumHomepageItem) {
  let reason = ''
  try {
    const { value } = await ElMessageBox.prompt(
      `Block ${item.forumName}'s custom homepage. Please provide a reason`,
      'Block Forum Homepage',
      {
        inputPlaceholder: 'Block reason (5-500 characters)',
        confirmButtonText: 'Block',
        cancelButtonText: 'Cancel',
        inputValidator: (v: string) => {
          const len = (v || '').trim().length
          return len >= 5 && len <= 500 ? true : 'Reason must be 5-500 characters'
        },
      }
    )
    reason = value.trim()
  } catch {
    return
  }
  try {
    await adminApi.blockForumHomepage(item.forumId, reason)
    ElMessage.success('Blocked')
    loadForumHomepages()
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to block'))
  }
}

async function unblockForumHomepage(item: AdminForumHomepageItem) {
  try {
    await ElMessageBox.confirm(
      `Restore ${item.forumName}'s custom homepage? It will be public again immediately.`,
      'Restore Forum Homepage',
      { confirmButtonText: 'Restore', cancelButtonText: 'Cancel', type: 'warning' }
    )
  } catch {
    return
  }
  try {
    await adminApi.unblockForumHomepage(item.forumId)
    ElMessage.success('Restored')
    loadForumHomepages()
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to restore'))
  }
}

async function approveAuth(item: AdminAuthItem) {
  try {
    await adminApi.approveAuthentication(item.id)
    ElMessage.success('Approved')
    loadAuth()
  } catch (e) {
    ElMessage.error(errorMessage(e))
  }
}

async function rejectAuth(item: AdminAuthItem) {
  try {
    const { value } = await ElMessageBox.prompt('Please enter a rejection reason', 'Reject Verification', {
      inputPlaceholder: 'Rejection reason',
      confirmButtonText: 'OK',
      cancelButtonText: 'Cancel',
    })
    await adminApi.rejectAuthentication(item.id, value)
    ElMessage.success('Rejected')
    loadAuth()
  } catch {
    /* cancelled or failed */
  }
}

onMounted(async () => {
  if (auth.isAuthenticated) await auth.fetchMe()
  isAdmin.value = auth.isAdmin
  ready.value = true
  if (!isAdmin.value) {
    ElMessage.error('Only admins can access the admin console')
    router.replace('/')
    return
  }
  loadAuth()
  loadForumHomepages()
  loadHomepages()
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
