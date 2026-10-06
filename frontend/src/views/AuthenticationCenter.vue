<template>
  <div>
    <h1 class="page-title">认证中心</h1>
    <p style="color: var(--of-text-muted); margin-bottom: 20px">
      完成认证后可解锁设置了门槛的子论坛。等级越高、证据越强：低-填写信息，中-上传证件，高-视频活体+证件。
    </p>

    <h2 class="section-title">我的认证记录</h2>
    <el-skeleton v-if="recordsLoading" :rows="3" animated />
    <div v-else-if="!records.length" class="state-box">还没有提交过任何认证</div>
    <div v-else class="record-list">
      <div v-for="record in records" :key="record.id" class="record-item">
        <div>
          <div style="font-weight: 600">
            {{ authTypeLabel(record.type) }}
            <el-tag v-if="record.level" size="small" type="info" effect="plain">
              {{ authLevelLabel(record.level) }}等级
            </el-tag>
          </div>
          <div class="sidebar-item-meta">
            提交于 {{ formatDate(record.createdAt) }}
          </div>
          <div
            v-if="record.status === 'REJECTED' && reasonOf(record)"
            class="record-reason"
          >
            驳回原因：{{ reasonOf(record) }}
          </div>
        </div>
        <div style="display: flex; align-items: center; gap: 10px">
          <el-tag :type="statusTag(record.status)" effect="light">
            {{ statusLabel(record.status) }}
          </el-tag>
          <el-button
            v-if="record.status === 'REJECTED'"
            size="small"
            :loading="retryingId === record.id"
            @click="retry(record)"
          >
            重新提交
          </el-button>
        </div>
      </div>
    </div>

    <h2 class="section-title">可提交的认证类型</h2>
    <el-skeleton v-if="typesLoading" :rows="4" animated />
    <div v-else-if="!types.length" class="state-box">暂无可用的认证类型</div>
    <div v-else class="auth-grid">
      <div v-for="meta in types" :key="meta.type" class="auth-card">
        <div class="auth-card-head">
          <h3>{{ meta.label }}</h3>
          <div class="auth-card-tags">
            <el-tag
              v-if="recordMap[meta.type]?.level"
              type="info"
              effect="plain"
            >
              已达 {{ authLevelLabel(recordMap[meta.type].level) }}
            </el-tag>
            <el-tag
              v-if="recordMap[meta.type]"
              :type="statusTag(recordMap[meta.type].status)"
              effect="light"
            >
              {{ statusLabel(recordMap[meta.type].status) }}
            </el-tag>
          </div>
        </div>
        <p v-if="meta.description" class="auth-card-desc">{{ meta.description }}</p>
        <div
          v-if="recordMap[meta.type]?.status === 'REJECTED' && reasonOf(recordMap[meta.type])"
          class="record-reason"
        >
          驳回原因：{{ reasonOf(recordMap[meta.type]) }}
        </div>
        <template v-if="canSubmit(meta.type)">
          <div class="auth-level-picker">
            <span class="auth-level-label">认证等级：</span>
            <el-radio-group v-model="selectedLevels[meta.type]" size="small">
              <el-radio-button
                v-for="lv in levelsOf(meta)"
                :key="lv.level"
                :value="lv.level"
                :disabled="levelDisabled(meta.type, lv.level)"
              >
                {{ lv.label }}
              </el-radio-button>
            </el-radio-group>
            <span class="auth-extreme-note" title="极高等级需通过移动 App 完成">极高需 App</span>
          </div>
          <div class="form-actions" style="margin-top: 12px">
            <el-button type="primary" @click="goWizard(meta.type)">
              {{ submitLabel(meta.type) }}
            </el-button>
          </div>
        </template>
        <p v-else class="auth-card-desc">
          {{
            recordMap[meta.type]?.status === 'APPROVED'
              ? '该认证已达最高等级。'
              : '该认证正在审核中，请耐心等待。'
          }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { authenticationApi } from '@/api'
import type { AuthLevel, AuthLevelMeta, AuthRecord, AuthStatus, AuthTypeMeta } from '@/types'
import {
  AUTH_TYPE_LABELS,
  AUTH_TYPE_ORDER,
  authLevelLabel,
  authTypeLabel,
  errorMessage,
  formatDate,
} from '@/utils/format'

const router = useRouter()

const LEVEL_RANK: Record<AuthLevel, number> = { LOW: 1, MEDIUM: 2, HIGH: 3, EXTREME: 4 }

const types = ref<AuthTypeMeta[]>([])
const records = ref<AuthRecord[]>([])
const typesLoading = ref(false)
const recordsLoading = ref(false)
const retryingId = ref<number | null>(null)
const selectedLevels = ref<Record<string, AuthLevel>>({})

const recordMap = computed<Record<string, AuthRecord>>(() => {
  const map: Record<string, AuthRecord> = {}
  for (const r of records.value) {
    if (r.type && !map[r.type]) map[r.type] = r
  }
  return map
})

function normalizeTypes(payload: unknown): AuthTypeMeta[] {
  let list: AuthTypeMeta[] = []
  if (Array.isArray(payload)) {
    list = payload
      .map((item) => item as Record<string, unknown>)
      .map((item) => ({
        type: String(item.type ?? item.key ?? item.code ?? ''),
        label: String(
          item.label ??
            item.name ??
            AUTH_TYPE_LABELS[String(item.type ?? item.key ?? item.code ?? '')] ??
            ''
        ),
        description: (item.description as string) || '',
        fields: Array.isArray(item.fields) ? (item.fields as AuthTypeMeta['fields']) : [],
        levels: Array.isArray(item.levels) ? (item.levels as AuthLevelMeta[]) : [],
      }))
      .filter((t) => !!t.type)
  } else if (payload && typeof payload === 'object') {
    list = Object.entries(payload as Record<string, unknown>).map(([key, value]) => {
      const val = (value || {}) as Record<string, unknown>
      return {
        type: String(val.type ?? key),
        label: String(val.label ?? AUTH_TYPE_LABELS[key] ?? key),
        description: (val.description as string) || '',
        fields: Array.isArray(val.fields) ? (val.fields as AuthTypeMeta['fields']) : [],
        levels: Array.isArray(val.levels) ? (val.levels as AuthLevelMeta[]) : [],
      }
    })
  }
  const order = new Map(AUTH_TYPE_ORDER.map((t, i) => [t, i]))
  return list.sort((a, b) => (order.get(a.type) ?? 99) - (order.get(b.type) ?? 99))
}

function statusLabel(status: AuthStatus): string {
  return status === 'APPROVED' ? '已通过' : status === 'PENDING' ? '审核中' : '已驳回'
}

function statusTag(status: AuthStatus): 'success' | 'warning' | 'danger' {
  return status === 'APPROVED' ? 'success' : status === 'PENDING' ? 'warning' : 'danger'
}

function reasonOf(record?: AuthRecord): string {
  return record?.rejectionReason || record?.reason || ''
}

const FALLBACK_LEVELS: { level: AuthLevel; label: string }[] = [
  { level: 'LOW', label: '低' },
  { level: 'MEDIUM', label: '中' },
  { level: 'HIGH', label: '高' },
  { level: 'EXTREME', label: '极高' },
]

function levelsOf(meta: AuthTypeMeta): { level: AuthLevel; label: string }[] {
  if (meta.levels?.length) {
    return meta.levels.map((l) => ({ level: l.level, label: l.label }))
  }
  return FALLBACK_LEVELS
}

function nextLevel(level: AuthLevel): AuthLevel | null {
  if (level === 'LOW') return 'MEDIUM'
  if (level === 'MEDIUM') return 'HIGH'
  return null
}

/** EXTREME 需移动 App；已通过时不允许选择不高于当前等级的等级 */
function levelDisabled(type: string, level: AuthLevel): boolean {
  if (level === 'EXTREME') return true
  const record = recordMap.value[type]
  if (record?.status === 'APPROVED') {
    return LEVEL_RANK[level] <= LEVEL_RANK[record.level || 'LOW']
  }
  return false
}

function defaultLevel(type: string): AuthLevel {
  const record = recordMap.value[type]
  if (!record) return 'LOW'
  if (record.status === 'APPROVED') {
    return nextLevel(record.level || 'LOW') ?? record.level ?? 'LOW'
  }
  return record.level || 'LOW'
}

/** 无记录 / 已驳回 / 已通过但可升级时允许提交 */
function canSubmit(type: string): boolean {
  const record = recordMap.value[type]
  if (!record) return true
  if (record.status === 'REJECTED') return true
  if (record.status === 'PENDING') return false
  return nextLevel(record.level || 'LOW') !== null
}

function submitLabel(type: string): string {
  const record = recordMap.value[type]
  if (!record) return '开始认证'
  if (record.status === 'REJECTED') return '重新提交'
  if (record.status === 'APPROVED') return '升级认证'
  return '提交认证'
}

function goWizard(type: string) {
  const level = selectedLevels.value[type] || 'LOW'
  router.push({ name: 'auth-level-wizard', params: { type, level } })
}

function ensureSelections() {
  for (const meta of types.value) {
    const current = selectedLevels.value[meta.type]
    if (!current || levelDisabled(meta.type, current)) {
      selectedLevels.value[meta.type] = defaultLevel(meta.type)
    }
  }
}

async function loadTypes() {
  typesLoading.value = true
  try {
    const data = await authenticationApi.types()
    types.value = normalizeTypes(data)
  } catch (e) {
    ElMessage.error(errorMessage(e, '认证类型加载失败'))
  } finally {
    typesLoading.value = false
  }
}

async function loadRecords() {
  recordsLoading.value = true
  try {
    const data = await authenticationApi.mine()
    records.value = Array.isArray(data) ? data : []
  } catch (e) {
    ElMessage.error(errorMessage(e, '认证记录加载失败'))
  } finally {
    recordsLoading.value = false
  }
}

async function retry(record: AuthRecord) {
  retryingId.value = record.id
  try {
    await authenticationApi.retry(record.id)
    ElMessage.success('已重新提交')
    await loadRecords()
  } catch (e) {
    ElMessage.error(errorMessage(e, '重新提交失败'))
  } finally {
    retryingId.value = null
  }
}

watch([types, records], ensureSelections, { immediate: true })

onMounted(() => {
  loadTypes()
  loadRecords()
})
</script>

<style scoped>
.auth-card-tags {
  display: flex;
  gap: 6px;
  align-items: center;
}
.auth-level-picker {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 8px;
}
.auth-level-label {
  font-size: 13px;
  color: var(--of-text-muted);
}
.auth-extreme-note {
  font-size: 12px;
  color: var(--of-text-muted);
}
</style>
