<template>
  <div class="page-grid no-right">
    <LeftRail />

    <div class="feed-column">
      <div v-reveal class="content-card info-head">
        <div>
          <span class="of-eyebrow">Account</span>
          <h1 class="page-title" style="margin: 0">Profile &amp; Verification</h1>
          <p class="info-desc">
            Your profile is visible only to you. After filling it in, you can raise your verification level as needed to meet the viewing and interaction thresholds of different forums.
          </p>
        </div>
        <div class="info-progress">
          <el-progress
            type="circle"
            :percentage="info?.completeness.percent ?? 0"
            :width="72"
          />
          <span class="muted">
            Completed {{ info?.completeness.filled ?? 0 }}/{{ info?.completeness.total ?? 0 }} fields
          </span>
        </div>
      </div>

      <el-skeleton v-if="loading" :rows="8" animated />
      <div v-else-if="error" class="state-box error">
        {{ error }}
        <div style="margin-top: 12px">
          <el-button size="small" @click="load">Retry</el-button>
        </div>
      </div>

      <template v-else>
        <section
          v-for="(group, gi) in info?.groups || []"
          :key="group.groupKey"
          v-reveal="Math.min(gi, 5) * 50"
          class="content-card info-group"
        >
          <h2 class="info-group-title">{{ group.groupLabel }}</h2>
          <div v-for="item in group.items" :key="item.fieldKey" class="info-item">
            <div class="info-item-main">
              <div class="info-item-label">
                <span>{{ item.label }}</span>
                <el-tag
                  v-if="item.authStatus === 'PENDING'"
                  size="small"
                  type="warning"
                  effect="light"
                >
                  Under review
                </el-tag>
                <el-tag
                  v-else-if="item.authLevel && rankOf(item.authLevel) > 0"
                  size="small"
                  type="success"
                  effect="plain"
                >
                  Verified {{ authLevelLabel(item.authLevel) }}
                </el-tag>
              </div>
              <div class="info-item-value" :class="{ empty: !item.valueText }">
                {{ item.valueText || 'Not provided' }}
              </div>
            </div>
            <div class="info-item-actions">
              <el-button size="small" text type="primary" @click="openEdit(item)">
                {{ item.valueText ? 'Edit' : 'Add' }}
              </el-button>
              <el-button
                v-if="authTypeOf(item.fieldKey)"
                size="small"
                text
                :disabled="item.authStatus === 'PENDING'"
                @click="goAuth(item)"
              >
                {{ item.authStatus === 'PENDING' ? 'Under review' : item.valueText ? 'Verify / Upgrade' : 'Verify' }}
              </el-button>
            </div>
          </div>
        </section>
      </template>
    </div>
  </div>

  <el-dialog v-model="editVisible" :title="`Edit ${editField?.label || ''}`" width="520px">
    <el-form v-if="editField" label-position="top">
      <template v-if="editField.fieldKey === 'REAL_NAME'">
        <el-form-item label="Full name" required>
          <el-input v-model="editData.fullName" maxlength="50" />
        </el-form-item>
      </template>

      <template v-else-if="editField.fieldKey === 'GENDER'">
        <el-form-item label="Gender" required>
          <el-select v-model="editData.gender" style="width: 100%">
            <el-option label="Male" value="MALE" />
            <el-option label="Female" value="FEMALE" />
          </el-select>
        </el-form-item>
      </template>

      <template v-else-if="editField.fieldKey === 'AGE'">
        <el-form-item label="Date of birth" required>
          <el-date-picker
            v-model="editData.birthDate"
            type="date"
            value-format="YYYY-MM-DD"
            placeholder="Select date of birth"
            style="width: 100%"
          />
        </el-form-item>
      </template>

      <template v-else-if="editField.fieldKey === 'COUNTRY'">
        <el-form-item label="Country/Region" required>
          <el-select v-model="editData.country" filterable style="width: 100%">
            <el-option
              v-for="o in optionsOf('COUNTRY')"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
        </el-form-item>
      </template>

      <template v-else-if="editField.fieldKey === 'CITY'">
        <el-form-item label="Country/Region">
          <el-select v-model="editData.country" filterable style="width: 100%">
            <el-option
              v-for="o in optionsOf('COUNTRY')"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="City" required>
          <el-input v-model="editData.city" maxlength="40" placeholder="e.g. Shanghai" />
        </el-form-item>
      </template>

      <template v-else-if="editField.fieldKey === 'OCCUPATION'">
        <el-form-item label="Occupation category" required>
          <el-select v-model="editData.category" style="width: 100%">
            <el-option
              v-for="o in optionsOf('OCCUPATION')"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="Job title (optional)">
          <el-input v-model="editData.title" maxlength="50" />
        </el-form-item>
      </template>

      <template v-else-if="editField.fieldKey === 'EDUCATION'">
        <el-form-item label="Education" required>
          <el-select v-model="editData.education" style="width: 100%">
            <el-option
              v-for="o in optionsOf('EDUCATION')"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
        </el-form-item>
      </template>

      <template v-else-if="editField.fieldKey === 'PROPERTY_AREA'">
        <el-form-item label="Property city">
          <el-select
            v-model="editData.cities"
            multiple
            filterable
            allow-create
            default-first-option
            placeholder="Type a city and press Enter; you can add multiple"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item label="Property district">
          <el-select
            v-model="editData.districts"
            multiple
            filterable
            allow-create
            default-first-option
            placeholder="Type a district and press Enter; you can add multiple"
            style="width: 100%"
          />
        </el-form-item>
      </template>

      <template v-else-if="editField.fieldKey === 'PROPERTY_COUNT'">
        <el-form-item label="Number of properties" required>
          <el-input-number v-model="editData.count" :min="0" :max="100" />
        </el-form-item>
      </template>

      <template v-else-if="editField.fieldKey === 'INTERESTS'">
        <el-form-item label="Interest tags">
          <el-select
            v-model="editData.tags"
            multiple
            filterable
            allow-create
            default-first-option
            placeholder="Select or enter interests"
            style="width: 100%"
          >
            <el-option
              v-for="o in optionsOf('INTERESTS')"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
        </el-form-item>
      </template>

      <el-alert
        v-if="editField.authable"
        type="info"
        :closable="false"
        show-icon
        title="Editing verified information will invalidate the current verification level and require re-verification."
        style="margin-top: 6px"
      />
    </el-form>
    <template #footer>
      <el-button @click="editVisible = false">Cancel</el-button>
      <el-button type="primary" :loading="saving" @click="save">Save</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import LeftRail from '@/components/LeftRail.vue'
import { infoApi } from '@/api'
import type { InfoField, InfoFieldGroup, UserInfoItem, UserInfoResponse } from '@/types'
import { authLevelLabel } from '@/utils/format'

const router = useRouter()

const info = ref<UserInfoResponse | null>(null)
const fields = ref<InfoFieldGroup[]>([])
const loading = ref(true)
const error = ref('')

const editVisible = ref(false)
const editField = ref<InfoField | null>(null)
const saving = ref(false)

const editData = reactive({
  fullName: '',
  gender: '',
  birthDate: '',
  country: 'CN',
  city: '',
  category: '',
  title: '',
  education: '',
  cities: [] as string[],
  districts: [] as string[],
  count: 0,
  tags: [] as string[],
})

const AUTH_TYPE_BY_FIELD: Record<string, string> = {
  REAL_NAME: 'REAL_NAME',
  GENDER: 'GENDER',
  AGE: 'AGE',
  COUNTRY: 'NATIONALITY',
  CITY: 'CITY',
  OCCUPATION: 'OCCUPATION',
  EDUCATION: 'EDUCATION',
  PROPERTY_AREA: 'PROPERTY_CERT',
  PROPERTY_COUNT: 'PROPERTY_CERT',
}

const LEVEL_RANK: Record<string, number> = { LOW: 1, MEDIUM: 2, HIGH: 3, EXTREME: 4 }

function flatFields(): Record<string, InfoField> {
  const map: Record<string, InfoField> = {}
  for (const group of fields.value) {
    for (const f of group.fields || []) map[f.fieldKey] = f
  }
  return map
}

function optionsOf(fieldKey: string): { value: string; label: string }[] {
  const field = flatFields()[fieldKey]
  return (field?.options || []).slice().sort((a, b) => (a.order ?? 0) - (b.order ?? 0))
}

function rankOf(level?: string | null): number {
  return level ? LEVEL_RANK[level] || 0 : 0
}

function authTypeOf(fieldKey: string): string | undefined {
  const field = flatFields()[fieldKey]
  if (!field?.authable) return undefined
  return AUTH_TYPE_BY_FIELD[fieldKey]
}

function resetEditData() {
  editData.fullName = ''
  editData.gender = ''
  editData.birthDate = ''
  editData.country = 'CN'
  editData.city = ''
  editData.category = ''
  editData.title = ''
  editData.education = ''
  editData.cities = []
  editData.districts = []
  editData.count = 0
  editData.tags = []
}

function openEdit(item: UserInfoItem) {
  const field = flatFields()[item.fieldKey]
  if (!field) return
  resetEditData()
  const value = (item.value || {}) as Record<string, unknown>
  editData.fullName = String(value.fullName ?? '')
  editData.gender = String(value.gender ?? '')
  editData.birthDate = String(value.birthDate ?? '')
  editData.country = String(value.country ?? 'CN')
  editData.city = String(value.city ?? '')
  editData.category = String(value.category ?? '')
  editData.title = String(value.title ?? '')
  editData.education = String(value.education ?? '')
  editData.cities = Array.isArray(value.cities) ? (value.cities as string[]) : []
  editData.districts = Array.isArray(value.districts) ? (value.districts as string[]) : []
  editData.count = typeof value.count === 'number' ? value.count : 0
  editData.tags = Array.isArray(value.tags) ? (value.tags as string[]) : []
  editField.value = field
  editVisible.value = true
}

function payloadOf(fieldKey: string): Record<string, unknown> | null {
  switch (fieldKey) {
    case 'REAL_NAME':
      if (!editData.fullName.trim()) return null
      return { fullName: editData.fullName.trim() }
    case 'GENDER':
      if (!editData.gender) return null
      return { gender: editData.gender }
    case 'AGE':
      if (!editData.birthDate) return null
      return { birthDate: editData.birthDate }
    case 'COUNTRY':
      if (!editData.country) return null
      return { country: editData.country }
    case 'CITY':
      if (!editData.city.trim()) return null
      return { country: editData.country || 'CN', city: editData.city.trim() }
    case 'OCCUPATION':
      if (!editData.category) return null
      return { category: editData.category, title: editData.title.trim() }
    case 'EDUCATION':
      if (!editData.education) return null
      return { education: editData.education }
    case 'PROPERTY_AREA':
      if (!editData.cities.length && !editData.districts.length) return null
      return { cities: editData.cities, districts: editData.districts }
    case 'PROPERTY_COUNT':
      if (typeof editData.count !== 'number') return null
      return { count: editData.count }
    case 'INTERESTS':
      return { tags: editData.tags }
    default:
      return null
  }
}

async function save() {
  if (!editField.value) return
  const payload = payloadOf(editField.value.fieldKey)
  if (!payload) {
    ElMessage.warning('Please fill in all required information')
    return
  }
  saving.value = true
  try {
    await infoApi.update(editField.value.fieldKey, payload)
    ElMessage.success('Saved')
    editVisible.value = false
    await load()
  } catch (e) {
    ElMessage.error(e instanceof Error ? e.message : 'Failed to save')
  } finally {
    saving.value = false
  }
}

function goAuth(item: UserInfoItem) {
  const type = authTypeOf(item.fieldKey)
  if (!type) return
  const current = item.authLevel || ''
  const next = current === 'MEDIUM' ? 'HIGH' : current === 'LOW' ? 'MEDIUM' : 'LOW'
  router.push({
    name: 'auth-level-wizard',
    params: { type, level: next },
  })
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    const [fieldGroups, mine] = await Promise.all([infoApi.fields(), infoApi.me()])
    fields.value = fieldGroups || []
    info.value = mine
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load information'
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>

<style scoped>
.info-head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  margin-bottom: 14px;
}
.info-desc {
  margin: 8px 0 0;
  color: var(--of-text-muted);
  font-size: 13px;
  line-height: 1.6;
}
.info-progress {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  min-width: 132px;
}
.info-progress .muted {
  white-space: nowrap;
  font-size: 12px;
}
.info-group {
  margin-bottom: 14px;
}
.info-group-title {
  margin: 0 0 10px;
  font-size: 15px;
}
.info-item {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 0;
  border-top: 1px solid var(--of-border);
}
.info-item:first-of-type {
  border-top: 0;
}
.info-item-label {
  display: flex;
  gap: 8px;
  align-items: center;
  font-weight: 500;
}
.info-item-value {
  margin-top: 4px;
  color: var(--of-text);
  font-size: 13px;
  word-break: break-all;
}
.info-item-value.empty {
  color: var(--of-text-subtle);
}
.info-item-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}
.muted {
  color: var(--of-text-muted);
  font-size: 12px;
}
</style>
