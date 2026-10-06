<template>
  <div class="content-card" style="max-width: 760px; margin: 0 auto">
    <h1 class="page-title">创建子论坛</h1>

    <el-form label-position="top">
      <el-form-item label="名称" required>
        <el-input v-model="form.name" maxlength="60" placeholder="例如：前端技术交流" />
      </el-form-item>

      <el-form-item label="Slug（URL 标识）" required>
        <el-input v-model="form.slug" placeholder="例如 frontend（小写字母、数字、连字符）">
          <template #append>
            <el-button @click="genSlug">自动生成</el-button>
          </template>
        </el-input>
      </el-form-item>

      <el-form-item label="简介">
        <el-input v-model="form.description" type="textarea" :rows="3" maxlength="200" show-word-limit />
      </el-form-item>

      <el-form-item label="图标（图片 URL，可选）">
        <el-input v-model="form.icon" placeholder="https://…" />
      </el-form-item>

      <el-form-item label="主题色">
        <el-color-picker v-model="form.themeColor" />
      </el-form-item>

      <el-form-item label="页面模板">
        <el-radio-group v-model="form.templateType">
          <el-radio value="DEFAULT">默认模板（公告 / 版块 / 链接）</el-radio>
          <el-radio value="CUSTOM">自定义 HTML 页面（需管理员审核）</el-radio>
        </el-radio-group>
      </el-form-item>

      <el-form-item label="互动门槛（需完成的认证）">
        <el-select
          v-model="form.requiredAuthTypes"
          multiple
          clearable
          placeholder="不选择表示无需认证"
          style="width: 100%"
        >
          <el-option
            v-for="t in authTypeOptions"
            :key="t.value"
            :label="t.label"
            :value="t.value"
          />
        </el-select>
      </el-form-item>

      <el-form-item label="最低年龄门槛（可选）">
        <el-input-number v-model="ageGate" :min="0" :max="120" :controls="false" />
        <span style="margin-left: 10px; color: var(--of-text-muted)">
          设置后需年龄 ≥ 该值的认证
        </span>
      </el-form-item>

      <el-form-item label="板块规则">
        <el-input v-model="form.rules" type="textarea" :rows="4" />
      </el-form-item>

      <div class="form-actions">
        <el-button type="primary" :loading="submitting" @click="submit">
          创建子论坛
        </el-button>
        <el-button @click="router.back()">取消</el-button>
      </div>
    </el-form>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { forumApi } from '@/api'
import { AUTH_TYPE_LABELS, AUTH_TYPE_ORDER, errorMessage } from '@/utils/format'

const router = useRouter()

const authTypeOptions = AUTH_TYPE_ORDER.map((t) => ({ label: AUTH_TYPE_LABELS[t], value: t }))

const form = reactive({
  name: '',
  slug: '',
  description: '',
  icon: '',
  themeColor: '#FF4500',
  templateType: 'DEFAULT',
  requiredAuthTypes: [] as string[],
  rules: '',
})
const ageGate = ref<number | undefined>(undefined)
const submitting = ref(false)

function genSlug() {
  const base = form.name.trim().toLowerCase()
  const slug = base
    .replace(/[^a-z0-9\u4e00-\u9fa5]+/g, '-')
    .replace(/^-+|-+$/g, '')
  form.slug = slug || `forum-${Date.now().toString(36)}`
}

async function submit() {
  if (!form.name.trim() || !form.slug.trim()) {
    ElMessage.warning('请填写名称和 Slug')
    return
  }
  if (!/^[a-z0-9-]+$/.test(form.slug.trim())) {
    ElMessage.warning('Slug 只能包含小写字母、数字和连字符')
    return
  }
  submitting.value = true
  try {
    const requiredAuthTypes = [...form.requiredAuthTypes]
    if (ageGate.value && ageGate.value > 0) requiredAuthTypes.push(`AGE_${ageGate.value}`)
    const created = await forumApi.create({
      name: form.name.trim(),
      slug: form.slug.trim(),
      description: form.description.trim(),
      icon: form.icon.trim(),
      themeColor: form.themeColor,
      templateType: form.templateType,
      rules: form.rules.trim(),
      requiredAuthTypes,
    })
    ElMessage.success('子论坛创建成功')
    router.push(`/f/${created.slug || form.slug}`)
  } catch (e) {
    ElMessage.error(errorMessage(e, '创建失败'))
  } finally {
    submitting.value = false
  }
}
</script>
