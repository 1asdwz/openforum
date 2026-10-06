<template>
  <el-form label-width="96px" label-position="left" @submit.prevent>
    <template v-if="hasFields">
      <el-form-item
        v-for="field in meta.fields"
        :key="field.name"
        :label="field.label || field.name"
        :required="!!field.required"
      >
        <el-select
          v-if="field.type === 'select' || (field.options && field.options.length)"
          v-model="model[field.name]"
          :placeholder="`请选择${field.label || field.name}`"
          style="width: 100%"
        >
          <el-option
            v-for="opt in normOptions(field.options)"
            :key="opt.value"
            :label="opt.label"
            :value="opt.value"
          />
        </el-select>

        <el-input-number
          v-else-if="isNumberField(field.type)"
          v-model="model[field.name]"
          :controls="false"
          style="width: 100%"
        />

        <el-date-picker
          v-else-if="field.type === 'date'"
          v-model="model[field.name]"
          type="date"
          value-format="YYYY-MM-DD"
          style="width: 100%"
        />

        <el-input
          v-else
          v-model="model[field.name]"
          :placeholder="`请输入${field.label || field.name}`"
        />
      </el-form-item>
    </template>

    <template v-else>
      <div v-for="(row, index) in rows" :key="index" class="kv-row">
        <el-input v-model="row.key" placeholder="字段名" style="width: 40%" />
        <el-input v-model="row.value" placeholder="字段值" style="width: 45%" />
        <el-button
          :disabled="rows.length <= 1"
          @click="rows.splice(index, 1)"
        >
          删除
        </el-button>
      </div>
      <el-button size="small" @click="rows.push({ key: '', value: '' })">
        + 添加字段
      </el-button>
    </template>

    <div v-if="!hideAction" class="form-actions" style="margin-top: 16px">
      <el-button type="primary" :loading="submitting" @click="onSubmit">
        提交认证
      </el-button>
    </div>
  </el-form>
</template>

<script setup lang="ts">
import { computed, reactive, watch } from 'vue'
import { ElMessage } from 'element-plus'
import type { AuthFieldDef, AuthTypeMeta } from '@/types'

const props = defineProps<{
  meta: AuthTypeMeta
  submitting?: boolean
  /** 隐藏内置提交按钮（由父组件控制提交时使用） */
  hideAction?: boolean
}>()

const emit = defineEmits<{
  (e: 'submit', data: Record<string, unknown>): void
  (e: 'change', data: Record<string, unknown>): void
}>()

const model = reactive<Record<string, unknown>>({})
const rows = reactive<{ key: string; value: string }[]>([{ key: '', value: '' }])

const hasFields = computed(() => (props.meta.fields?.length || 0) > 0)

function isNumberField(type?: string) {
  return type === 'number' || type === 'age' || type === 'int' || type === 'integer'
}

function normOptions(
  options?: AuthFieldDef['options']
): { label: string; value: string }[] {
  if (!options) return []
  return options.map((o) =>
    typeof o === 'string' ? { label: o, value: o } : { label: o.label, value: o.value }
  )
}

function initModel() {
  Object.keys(model).forEach((k) => delete model[k])
  props.meta.fields?.forEach((f) => {
    model[f.name] = isNumberField(f.type) || f.type === 'date' ? undefined : ''
  })
}

watch(
  () => props.meta,
  () => {
    initModel()
    rows.splice(0, rows.length, { key: '', value: '' })
  },
  { immediate: true }
)

/** 收集当前表单数据 */
function collect(): Record<string, unknown> {
  if (hasFields.value) return { ...model }
  const data: Record<string, unknown> = {}
  for (const row of rows) {
    const key = row.key.trim()
    if (key) data[key] = row.value
  }
  return data
}

/** 校验表单，返回错误信息（null 表示通过） */
function validate(): string | null {
  if (hasFields.value) {
    for (const f of props.meta.fields) {
      if (f.required) {
        const v = model[f.name]
        if (v === undefined || v === null || v === '') {
          return `请填写「${f.label || f.name}」`
        }
      }
    }
    return null
  }
  return Object.keys(collect()).length ? null : '请至少填写一个字段'
}

function onSubmit() {
  const err = validate()
  if (err) {
    ElMessage.warning(err)
    return
  }
  emit('submit', collect())
}

watch(model, () => emit('change', collect()), { deep: true })
watch(rows, () => emit('change', collect()), { deep: true })

defineExpose({ getData: collect, validate })
</script>
