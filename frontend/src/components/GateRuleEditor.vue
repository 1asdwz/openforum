<template>
  <div class="gate-editor">
    <div v-if="!conditions.length" class="gate-empty">
      未设置条件，任何人（含未登录用户）均可通过。
    </div>

    <div v-for="(c, i) in conditions" :key="i" class="gate-row">
      <el-select
        v-model="c.field"
        class="gate-field"
        placeholder="信息项"
        @change="onFieldChange(c)"
      >
        <el-option
          v-for="f in usableFields"
          :key="f.fieldKey"
          :label="f.label"
          :value="f.fieldKey"
        />
      </el-select>

      <el-select v-model="c.op" class="gate-op" placeholder="条件" @change="onOpChange(c)">
        <el-option
          v-for="op in opsOf(c.field)"
          :key="op"
          :label="opLabel(op)"
          :value="op"
        />
      </el-select>

      <div class="gate-value">
        <template v-if="c.op === 'in' || c.op === 'not_in'">
          <el-select
            v-model="c.values"
            multiple
            filterable
            :allow-create="allowCreate(c.field)"
            default-first-option
            placeholder="选择或输入取值"
            style="width: 100%"
            @change="syncLabels(c)"
          >
            <el-option
              v-for="o in optionsOf(c.field)"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
        </template>

        <template v-else-if="isRangeField(c.field) && c.op === 'between'">
          <div class="gate-range">
            <el-input-number v-model="c.min" :controls="false" :min="0" placeholder="下限" />
            <span>—</span>
            <el-input-number v-model="c.max" :controls="false" :min="0" placeholder="上限" />
          </div>
        </template>

        <template v-else-if="isRangeField(c.field)">
          <el-input-number
            v-model="c[c.op === 'lte' ? 'max' : 'min']"
            :controls="false"
            :min="0"
            placeholder="数值"
          />
        </template>

        <template v-else-if="c.field === 'EDUCATION'">
          <el-select v-model="c.value" placeholder="学历" style="width: 100%">
            <el-option
              v-for="o in optionsOf(c.field)"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
        </template>

        <el-input v-else v-model="c.value" placeholder="取值" />
      </div>

      <el-select
        v-if="authableOf(c.field)"
        v-model="c.authLevel"
        class="gate-auth"
        clearable
        placeholder="无认证等级要求"
      >
        <el-option label="认证 ≥ 低" value="LOW" />
        <el-option label="认证 ≥ 中" value="MEDIUM" />
        <el-option label="认证 ≥ 高" value="HIGH" />
        <el-option label="认证 ≥ 极高" value="EXTREME" />
      </el-select>

      <el-button text type="danger" @click="removeCondition(i)">删除</el-button>
    </div>

    <el-button size="small" @click="addCondition">+ 添加条件</el-button>
    <p class="gate-hint">
      多个条件需同时满足；同一条件内多选取值满足其一即可。操作门槛会自动叠加浏览门槛。
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { GateCondition, GateRule, InfoField } from '@/types'

const props = defineProps<{
  modelValue?: GateRule | null
  fields: InfoField[]
}>()

const emit = defineEmits<{ 'update:modelValue': [GateRule | null] }>()

const conditions = ref<GateCondition[]>(normalize(props.modelValue))
let syncing = false

const usableFields = computed(() => props.fields.filter((f) => f.gateOps?.length))
const fieldMap = computed<Record<string, InfoField>>(() => {
  const map: Record<string, InfoField> = {}
  for (const f of usableFields.value) map[f.fieldKey] = f
  return map
})

function normalize(rule?: GateRule | null): GateCondition[] {
  return (rule?.conditions || []).map((c) => ({ ...c }))
}

function opsOf(field: string): string[] {
  return fieldMap.value[field]?.gateOps || []
}

function authableOf(field: string): boolean {
  return !!fieldMap.value[field]?.authable
}

function isRangeField(field: string): boolean {
  return field === 'AGE' || field === 'PROPERTY_COUNT'
}

function allowCreate(field: string): boolean {
  return field === 'CITY' || field === 'PROPERTY_AREA'
}

function optionsOf(field: string): { value: string; label: string }[] {
  const options = fieldMap.value[field]?.options || []
  return [...options].sort((a, b) => (a.order ?? 0) - (b.order ?? 0))
}

function opLabel(op: string): string {
  switch (op) {
    case 'in':
      return '属于'
    case 'not_in':
      return '不属于'
    case 'between':
      return '介于'
    case 'gte':
      return '≥'
    case 'lte':
      return '≤'
    case 'eq':
      return '等于'
    default:
      return op
  }
}

function firstField(): string {
  return usableFields.value[0]?.fieldKey || 'GENDER'
}

function defaultCondition(): GateCondition {
  const field = firstField()
  return {
    field,
    op: opsOf(field)[0] || 'in',
    values: [],
    labels: [],
  }
}

function onFieldChange(c: GateCondition) {
  const ops = opsOf(c.field)
  c.op = ops[0]
  c.values = []
  c.labels = []
  c.value = ''
  c.min = undefined
  c.max = undefined
  c.authLevel = undefined
}

function onOpChange(c: GateCondition) {
  if (c.op !== 'in' && c.op !== 'not_in') {
    c.values = []
    c.labels = []
  }
  if (c.op !== 'between') {
    c.min = undefined
    c.max = undefined
  }
}

function syncLabels(c: GateCondition) {
  const map = new Map(optionsOf(c.field).map((o) => [o.value, o.label]))
  c.labels = (c.values || []).map((v) => map.get(v) || v)
}

function addCondition() {
  conditions.value.push(defaultCondition())
  emitRule()
}

function removeCondition(index: number) {
  conditions.value.splice(index, 1)
  emitRule()
}

function emitRule() {
  if (syncing) return
  const conditionsOut = conditions.value
    .filter((c) => c.field)
    .map((c) => {
      const out: GateCondition = { field: c.field }
      if (c.op) out.op = c.op
      if (c.op === 'in' || c.op === 'not_in') {
        out.values = c.values || []
        if (c.labels?.length) out.labels = c.labels
      } else if (isRangeField(c.field)) {
        if (c.op === 'between') {
          if (typeof c.min === 'number') out.min = c.min
          if (typeof c.max === 'number') out.max = c.max
        } else if (c.op === 'gte' && typeof c.min === 'number') {
          out.min = c.min
        } else if (c.op === 'lte' && typeof c.max === 'number') {
          out.max = c.max
        }
      } else if (c.field === 'EDUCATION') {
        if (c.value) out.value = c.value
      } else if (c.value) {
        out.value = c.value
      }
      if (c.authLevel) out.authLevel = c.authLevel
      return out
    })
  emit('update:modelValue', conditionsOut.length ? { version: 1, conditions: conditionsOut } : null)
}

watch(
  conditions,
  () => emitRule(),
  { deep: true }
)

watch(
  () => props.modelValue,
  (rule) => {
    const incoming = JSON.stringify(rule?.conditions || [])
    const current = JSON.stringify(conditions.value)
    if (incoming === current) return
    syncing = true
    conditions.value = normalize(rule)
    syncing = false
  }
)
</script>

<style scoped>
.gate-editor {
  width: 100%;
}
.gate-empty {
  padding: 10px 12px;
  border: 1px dashed var(--of-border);
  border-radius: 8px;
  color: var(--of-text-muted);
  font-size: 13px;
  margin-bottom: 8px;
}
.gate-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 10px;
  flex-wrap: wrap;
}
.gate-field {
  width: 130px;
}
.gate-op {
  width: 100px;
}
.gate-value {
  flex: 1;
  min-width: 180px;
}
.gate-auth {
  width: 160px;
}
.gate-range {
  display: flex;
  gap: 6px;
  align-items: center;
}
.gate-range :deep(.el-input-number) {
  flex: 1;
}
.gate-hint {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--of-text-subtle);
}
</style>
