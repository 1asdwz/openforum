<template>
  <div>
    <el-skeleton v-if="loading" :rows="8" animated />

    <div v-else-if="error" class="state-box error">
      {{ error }}
      <div style="margin-top: 12px">
        <el-button size="small" @click="loadForum">重试</el-button>
      </div>
    </div>

    <div v-else-if="forum && !isOwner" class="state-box error">
      你没有权限管理该子论坛
    </div>

    <template v-else-if="forum">
      <h1 class="page-title">管理 {{ forum.name }}</h1>

      <el-tabs v-model="activeTab">
        <el-tab-pane label="基本信息" name="basic">
          <el-form label-position="top">
            <el-form-item label="名称">
              <el-input v-model="form.name" />
            </el-form-item>
            <el-form-item label="简介">
              <el-input v-model="form.description" type="textarea" :rows="3" />
            </el-form-item>
            <el-form-item label="图标（图片 URL）">
              <el-input v-model="form.icon" />
            </el-form-item>
            <el-form-item label="主题色">
              <el-color-picker v-model="form.themeColor" />
            </el-form-item>
            <el-form-item label="页面模板">
              <el-radio-group v-model="form.templateType">
                <el-radio value="DEFAULT">默认模板</el-radio>
                <el-radio value="CUSTOM">自定义 HTML</el-radio>
              </el-radio-group>
            </el-form-item>
            <el-form-item label="浏览门槛">
              <GateRuleEditor v-model="form.browseRule" :fields="flatFields" />
            </el-form-item>
            <el-form-item label="操作门槛（发帖 / 评论 / 投票）">
              <GateRuleEditor v-model="form.operateRule" :fields="flatFields" />
            </el-form-item>
            <el-form-item label="板块规则">
              <el-input v-model="form.rules" type="textarea" :rows="4" />
            </el-form-item>
            <div class="form-actions">
              <el-button type="primary" :loading="saving" @click="saveBasic">
                保存修改
              </el-button>
            </div>
          </el-form>
        </el-tab-pane>

        <el-tab-pane label="默认模板" name="template">
          <el-alert
            v-if="form.templateType !== 'DEFAULT'"
            type="info"
            show-icon
            :closable="false"
            title="当前选择的是「自定义 HTML」页面，默认模板内容仅在切换为「默认模板」时对外展示"
            style="margin-bottom: 12px"
          />
          <el-form label-position="top">
            <el-form-item label="公告">
              <el-input
                v-model="form.announcement"
                type="textarea"
                :rows="3"
                placeholder="展示在子论坛页面顶部的公告"
              />
            </el-form-item>

            <el-form-item label="版块">
              <div v-for="(s, i) in form.sections" :key="i" class="template-row">
                <el-input v-model="s.title" placeholder="标题" style="width: 200px" />
                <el-input v-model="s.content" placeholder="内容" />
                <el-button text type="danger" @click="removeSection(i)">删除</el-button>
              </div>
              <el-button size="small" @click="addSection">+ 添加版块</el-button>
            </el-form-item>

            <el-form-item label="相关链接">
              <div v-for="(l, i) in form.links" :key="i" class="template-row">
                <el-input v-model="l.label" placeholder="名称" style="width: 180px" />
                <el-input v-model="l.url" placeholder="https://…" />
                <el-button text type="danger" @click="removeLink(i)">删除</el-button>
              </div>
              <el-button size="small" @click="addLink">+ 添加链接</el-button>
            </el-form-item>

            <div class="form-actions">
              <el-button type="primary" :loading="saving" @click="saveBasic">
                保存修改
              </el-button>
            </div>
          </el-form>
        </el-tab-pane>

        <el-tab-pane label="自定义页面" name="custom">
          <div class="settings-status">
            <span>当前状态：</span>
            <el-tag :type="customStatusTag(customStatus)" effect="light">
              {{ customStatusLabel(customStatus) }}
            </el-tag>
            <span v-if="customReviewNote" style="color: #d03050">
              审核意见：{{ customReviewNote }}
            </span>
          </div>

          <p style="color: var(--of-text-muted); margin-bottom: 10px">
            页面 HTML 会经后端白名单净化后保存，提交审核通过后对外展示（沙箱 iframe 渲染）。
          </p>

          <el-input
            v-model="customHtml"
            type="textarea"
            :rows="16"
            placeholder="粘贴自定义 HTML 片段，例如 <h1>欢迎</h1>"
          />

          <div class="form-actions">
            <el-button :loading="savingCustom" @click="saveCustom">
              保存草稿
            </el-button>
            <el-button
              type="primary"
              :loading="submittingCustom"
              :disabled="!customHtml.trim()"
              @click="submitCustom"
            >
              提交审核
            </el-button>
          </div>
        </el-tab-pane>
      </el-tabs>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import GateRuleEditor from '@/components/GateRuleEditor.vue'
import { forumApi, infoApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { CustomPageStatus, GateRule, InfoField, InfoFieldGroup, SubForum } from '@/types'
import { errorMessage, parseTemplateConfig } from '@/utils/format'

const route = useRoute()
const auth = useAuthStore()

const fieldGroups = ref<InfoFieldGroup[]>([])
const flatFields = computed<InfoField[]>(() =>
  fieldGroups.value.flatMap((g) => g.fields || [])
)

const forum = ref<SubForum | null>(null)
const loading = ref(true)
const error = ref('')
const activeTab = ref('basic')
const saving = ref(false)

const customHtml = ref('')
const customStatus = ref<CustomPageStatus | undefined>(undefined)
const customReviewNote = ref('')
const savingCustom = ref(false)
const submittingCustom = ref(false)

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

const isOwner = computed(
  () => !!forum.value && !!auth.user && forum.value.creatorId === auth.user.id
)

function customStatusLabel(status?: CustomPageStatus): string {
  switch (status) {
    case 'DRAFT':
      return '草稿'
    case 'PENDING':
      return '审核中'
    case 'APPROVED':
      return '已通过'
    case 'REJECTED':
      return '已驳回'
    default:
      return '未提交'
  }
}

function customStatusTag(status?: CustomPageStatus) {
  switch (status) {
    case 'APPROVED':
      return 'success'
    case 'PENDING':
      return 'warning'
    case 'REJECTED':
      return 'danger'
    default:
      return 'info'
  }
}

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

    if (isOwner.value && data.id) {
      await loadCustomPage(data.id)
    }
  } catch (e) {
    error.value = errorMessage(e, '子论坛加载失败')
  } finally {
    loading.value = false
  }
}

async function loadCustomPage(id: number) {
  try {
    const info = await forumApi.getCustomPage(id)
    customHtml.value = info?.html || ''
    customStatus.value = info?.status
    customReviewNote.value = info?.reviewNote || info?.customPageReviewNote || ''
  } catch {
    customHtml.value = forum.value?.customPageHtml || ''
    customStatus.value = forum.value?.customPageStatus
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
    ElMessage.success('已保存')
    await loadForum()
  } catch (e) {
    ElMessage.error(errorMessage(e, '保存失败'))
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

async function saveCustom() {
  if (!forum.value) return
  savingCustom.value = true
  try {
    const info = await forumApi.saveCustomPage(forum.value.id, customHtml.value)
    customStatus.value = info?.status || 'DRAFT'
    ElMessage.success('草稿已保存')
  } catch (e) {
    ElMessage.error(errorMessage(e, '保存失败'))
  } finally {
    savingCustom.value = false
  }
}

async function submitCustom() {
  if (!forum.value) return
  submittingCustom.value = true
  try {
    await forumApi.saveCustomPage(forum.value.id, customHtml.value)
    await forumApi.submitCustomPage(forum.value.id)
    customStatus.value = 'PENDING'
    ElMessage.success('已提交审核')
  } catch (e) {
    ElMessage.error(errorMessage(e, '提交失败'))
  } finally {
    submittingCustom.value = false
  }
}

onMounted(async () => {
  loadForum()
  try {
    fieldGroups.value = (await infoApi.fields()) || []
  } catch {
    // 目录加载失败时门槛编辑器为空，不影响其他管理功能
  }
})
</script>
