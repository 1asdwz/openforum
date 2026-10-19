<template>
  <div class="content-card" style="max-width: 760px; margin: 0 auto">
    <h1 class="page-title">Create forum</h1>

    <el-form label-position="top">
      <el-form-item label="Name" required>
        <el-input v-model="form.name" maxlength="60" placeholder="e.g. Frontend Tech Exchange" />
      </el-form-item>

      <el-form-item label="Slug (URL identifier)" required>
        <el-input v-model="form.slug" placeholder="e.g. frontend (lowercase letters, numbers, hyphens)">
          <template #append>
            <el-button @click="genSlug">Auto-generate</el-button>
          </template>
        </el-input>
      </el-form-item>

      <el-form-item label="Description">
        <el-input v-model="form.description" type="textarea" :rows="3" maxlength="200" show-word-limit />
      </el-form-item>

      <el-form-item label="Icon (image URL, optional)">
        <el-input v-model="form.icon" placeholder="https://…" />
      </el-form-item>

      <el-form-item label="Theme color">
        <el-color-picker v-model="form.themeColor" />
      </el-form-item>

      <el-form-item label="Page template">
        <el-radio-group v-model="form.templateType">
          <el-radio value="DEFAULT">Default template (announcement / sections / links)</el-radio>
          <el-radio value="CUSTOM">Custom HTML page (requires admin review)</el-radio>
        </el-radio-group>
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
        <el-button type="primary" :loading="submitting" @click="submit">
          Create forum
        </el-button>
        <el-button @click="router.back()">Cancel</el-button>
      </div>
    </el-form>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import GateRuleEditor from '@/components/GateRuleEditor.vue'
import { forumApi, infoApi } from '@/api'
import type { GateRule, InfoField, InfoFieldGroup } from '@/types'
import { errorMessage } from '@/utils/format'

const router = useRouter()

const fieldGroups = ref<InfoFieldGroup[]>([])
const flatFields = computed<InfoField[]>(() =>
  fieldGroups.value.flatMap((g) => g.fields || [])
)

const form = reactive({
  name: '',
  slug: '',
  description: '',
  icon: '',
  themeColor: '#FF4500',
  templateType: 'DEFAULT',
  browseRule: null as GateRule | null,
  operateRule: null as GateRule | null,
  rules: '',
})
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
    ElMessage.warning('Please enter a name and slug')
    return
  }
  if (!/^[a-z0-9-]+$/.test(form.slug.trim())) {
    ElMessage.warning('Slug can only contain lowercase letters, numbers, and hyphens')
    return
  }
  submitting.value = true
  try {
    const created = await forumApi.create({
      name: form.name.trim(),
      slug: form.slug.trim(),
      description: form.description.trim(),
      icon: form.icon.trim(),
      themeColor: form.themeColor,
      templateType: form.templateType,
      rules: form.rules.trim(),
      browseRule: form.browseRule,
      operateRule: form.operateRule,
    })
    ElMessage.success('Forum created successfully')
    router.push(`/f/${created.slug || form.slug}`)
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to create forum'))
  } finally {
    submitting.value = false
  }
}

onMounted(async () => {
  try {
    fieldGroups.value = (await infoApi.fields()) || []
  } catch {
    // If loading fields fails, the threshold editor stays empty; creation is unaffected
  }
})
</script>
