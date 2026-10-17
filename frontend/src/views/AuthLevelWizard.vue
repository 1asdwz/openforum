<template>
  <div class="auth-wizard">
    <h1 class="page-title">认证向导</h1>

    <div v-if="loadError" class="state-box">{{ loadError }}</div>
    <el-skeleton v-else-if="!meta" :rows="6" animated />

    <template v-else>
      <p class="wizard-subtitle">
        {{ meta.label }} · <strong>{{ authLevelLabel(level) }}等级</strong>
      </p>

      <el-steps :active="activeIndex" align-center finish-status="success" class="wizard-steps">
        <el-step v-for="s in steps" :key="s.key" :title="s.title" />
      </el-steps>

      <div class="wizard-body">
        <!-- 说明与注意事项 -->
        <section v-if="currentKey === 'intro'">
          <h3 class="section-title">{{ authLevelLabel(level) }}等级认证说明</h3>
          <p v-if="levelMeta?.requirements?.length" class="wizard-lead">本等级需要：</p>
          <ul class="wizard-list">
            <li v-for="(r, i) in levelMeta?.requirements || []" :key="i">{{ r }}</li>
          </ul>

          <h3 class="section-title">拍摄 / 操作注意事项</h3>
          <ul class="wizard-list">
            <li v-for="(t, i) in tips" :key="i">{{ t }}</li>
          </ul>

          <el-alert
            v-if="isExtreme"
            title="极高等级需要在「高」等级基础上，使用 OpenForum 移动 App 完成防伪录制，网页端暂不支持提交。"
            type="warning"
            :closable="false"
            show-icon
            style="margin-top: 12px"
          />
        </section>

        <!-- 信息填写 -->
        <section v-else-if="currentKey === 'info'">
          <h3 class="section-title">填写认证信息</h3>
          <AuthTypeForm
            ref="formRef"
            :meta="meta"
            hide-action
            @change="infoData = $event"
          />
        </section>

        <!-- 证件上传 -->
        <section v-else-if="currentKey === 'docs'">
          <h3 class="section-title">上传证件材料</h3>
          <DocumentUpload :min-count="1" :max-count="5" @change="documentIds = $event" />
        </section>

        <!-- 视频认证 -->
        <section v-else-if="currentKey === 'video'">
          <h3 class="section-title">视频认证（活体检测）</h3>
          <VideoCapture @done="videoResult = $event" />
        </section>

        <!-- 提交 -->
        <section v-else>
          <template v-if="isExtreme">
            <el-alert
              title="极高等级需通过 OpenForum 移动 App 完成防伪录制"
              description="请在 App 中完成人脸活体 + 证件 + App 防伪录制后，由 App 自动提交。网页端不提供极高等级的提交入口。"
              type="warning"
              :closable="false"
              show-icon
            />
          </template>

          <template v-else-if="result">
            <el-result
              :icon="result.status === 'APPROVED' ? 'success' : result.status === 'REJECTED' ? 'error' : 'warning'"
              :title="statusText(result.status)"
              :sub-title="result.reason || result.rejectionReason || ''"
            >
              <template #extra>
                <el-button type="primary" @click="goCenter">返回资料与认证</el-button>
              </template>
            </el-result>
          </template>

          <template v-else>
            <h3 class="section-title">确认并提交</h3>
            <ul class="wizard-summary">
              <li>认证类型：{{ meta.label }}</li>
              <li>认证等级：{{ authLevelLabel(level) }}</li>
              <li v-if="showDocs">证件材料：{{ documentIds.length }} 张</li>
              <li v-if="showVideo">视频认证：{{ videoResult ? (videoResult.liveness.degraded ? '已录制（降级转人工）' : '已完成活体检测') : '未完成' }}</li>
            </ul>
            <div class="wizard-actions">
              <el-button type="primary" :loading="submitting" @click="submit">提交认证</el-button>
            </div>
          </template>
        </section>
      </div>

      <div class="wizard-nav">
        <el-button :disabled="activeIndex === 0" @click="prev">上一步</el-button>
        <el-button
          v-if="activeIndex < steps.length - 1"
          type="primary"
          @click="next"
        >
          下一步
        </el-button>
        <el-button v-else @click="goCenter">返回资料与认证</el-button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import AuthTypeForm from '@/components/AuthTypeForm.vue'
import DocumentUpload from '@/components/DocumentUpload.vue'
import VideoCapture from '@/components/VideoCapture.vue'
import { authenticationApi } from '@/api'
import type {
  AuthLevel,
  AuthLevelMeta,
  AuthRecord,
  AuthStatus,
  AuthTypeMeta,
  AuthenticationSubmitPayload,
} from '@/types'
import type { VideoAuthResult } from '@/utils/liveness'
import { authLevelLabel, errorMessage } from '@/utils/format'

const route = useRoute()
const router = useRouter()

const LEVEL_RANK: Record<AuthLevel, number> = { LOW: 1, MEDIUM: 2, HIGH: 3, EXTREME: 4 }
const VALID_LEVELS: AuthLevel[] = ['LOW', 'MEDIUM', 'HIGH', 'EXTREME']

const type = computed(() => String(route.params.type || '').toUpperCase())
const level = computed<AuthLevel>(() => String(route.params.level || '').toUpperCase() as AuthLevel)

const meta = ref<AuthTypeMeta | null>(null)
const loadError = ref('')
const activeIndex = ref(0)

const infoData = ref<Record<string, unknown>>({})
const formRef = ref<{ getData: () => Record<string, unknown>; validate: () => string | null } | null>(null)
const documentIds = ref<number[]>([])
const videoResult = ref<VideoAuthResult | null>(null)
const submitting = ref(false)
const result = ref<AuthRecord | null>(null)

const rank = computed(() => LEVEL_RANK[level.value] ?? 0)
const showDocs = computed(() => rank.value >= 2)
const showVideo = computed(() => rank.value >= 3)
const isExtreme = computed(() => level.value === 'EXTREME')
const levelMeta = computed<AuthLevelMeta | null>(
  () => meta.value?.levels?.find((l) => l.level === level.value) || null
)

const DEFAULT_TIPS = [
  '光线充足、均匀，避免逆光或过曝',
  '面部无遮挡，不戴帽子、墨镜、口罩',
  '正对镜头，保持面部完整入镜',
  '按屏幕提示依次完成随机动作',
]

const tips = computed(() => levelMeta.value?.tips?.length ? levelMeta.value.tips : DEFAULT_TIPS)

type StepKey = 'intro' | 'info' | 'docs' | 'video' | 'submit'
const steps = computed<{ key: StepKey; title: string }[]>(() => {
  const s: { key: StepKey; title: string }[] = [{ key: 'intro', title: '说明' }]
  if (!isExtreme.value) s.push({ key: 'info', title: '信息填写' })
  if (showDocs.value) s.push({ key: 'docs', title: '证件上传' })
  if (showVideo.value) s.push({ key: 'video', title: '视频认证' })
  s.push({ key: 'submit', title: isExtreme.value ? '需移动 App' : '提交' })
  return s
})
const currentKey = computed<StepKey>(() => steps.value[activeIndex.value]?.key ?? 'intro')

function statusText(status: AuthStatus): string {
  return status === 'APPROVED' ? '认证已通过' : status === 'PENDING' ? '提交成功，等待审核' : '认证未通过'
}

function next() {
  const key = currentKey.value
  if (key === 'info') {
    const err = formRef.value?.validate()
    if (err) {
      ElMessage.warning(err)
      return
    }
    infoData.value = formRef.value?.getData() || {}
  }
  if (key === 'docs' && documentIds.value.length < 1) {
    ElMessage.warning('请至少上传 1 张证件照片')
    return
  }
  if (key === 'video' && !videoResult.value) {
    ElMessage.warning('请先完成视频认证')
    return
  }
  if (activeIndex.value < steps.value.length - 1) activeIndex.value += 1
}

function prev() {
  if (activeIndex.value > 0) activeIndex.value -= 1
}

async function submit() {
  submitting.value = true
  try {
    const payload: AuthenticationSubmitPayload = {
      type: type.value,
      level: level.value,
      data: infoData.value,
    }
    if (documentIds.value.length) payload.mediaIds = documentIds.value
    if (videoResult.value) {
      payload.videoMediaId = videoResult.value.videoMediaId
      payload.challengeId = videoResult.value.challengeId
      payload.liveness = videoResult.value.liveness as unknown as Record<string, unknown>
    }
    result.value = await authenticationApi.submit(payload)
    ElMessage.success('认证已提交')
  } catch (e) {
    ElMessage.error(errorMessage(e, '提交失败'))
  } finally {
    submitting.value = false
  }
}

function goCenter() {
  router.push('/profile')
}

async function load() {
  if (!type.value || !VALID_LEVELS.includes(level.value)) {
    loadError.value = '认证类型或等级参数无效'
    return
  }
  try {
    const list = await authenticationApi.types()
    meta.value = (list || []).find((m) => m.type === type.value) || null
    if (!meta.value) loadError.value = '未找到该认证类型'
  } catch (e) {
    loadError.value = errorMessage(e, '认证类型加载失败')
  }
}

onMounted(load)
</script>

<style scoped>
.wizard-subtitle {
  color: var(--of-text-muted);
  margin-bottom: 16px;
}
.wizard-steps {
  margin-bottom: 20px;
}
.wizard-lead {
  margin-bottom: 6px;
}
.wizard-list {
  margin: 0 0 16px;
  padding-left: 20px;
  color: var(--of-text);
}
.wizard-list li {
  line-height: 1.9;
}
.wizard-summary {
  margin: 0 0 16px;
  padding-left: 20px;
  line-height: 2;
}
.wizard-actions {
  margin-top: 12px;
}
.wizard-nav {
  margin-top: 24px;
  display: flex;
  gap: 10px;
}
</style>
