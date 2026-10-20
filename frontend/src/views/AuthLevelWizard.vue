<template>
  <div class="auth-wizard">
    <span class="of-eyebrow">Verification</span>
    <h1 class="page-title">Verification Wizard</h1>

    <div v-if="loadError" class="state-box">{{ loadError }}</div>
    <el-skeleton v-else-if="!meta" :rows="6" animated />

    <template v-else>
      <p class="wizard-subtitle">
        {{ meta.label }} · <strong>{{ authLevelLabel(level) }} level</strong>
      </p>

      <el-steps :active="activeIndex" align-center finish-status="success" class="wizard-steps">
        <el-step v-for="s in steps" :key="s.key" :title="s.title" />
      </el-steps>

      <div class="wizard-body">
        <!-- Overview and tips -->
        <section v-if="currentKey === 'intro'">
          <h3 class="section-title">{{ authLevelLabel(level) }} Level Verification</h3>
          <p v-if="levelMeta?.requirements?.length" class="wizard-lead">This level requires:</p>
          <ul class="wizard-list">
            <li v-for="(r, i) in levelMeta?.requirements || []" :key="i">{{ r }}</li>
          </ul>

          <h3 class="section-title">Photo / Action Tips</h3>
          <ul class="wizard-list">
            <li v-for="(t, i) in tips" :key="i">{{ t }}</li>
          </ul>

          <el-alert
            v-if="isExtreme"
            title="The Extreme level builds on the High level and requires anti-spoofing recording in the OpenForum mobile app. Web submissions are not supported yet."
            type="warning"
            :closable="false"
            show-icon
            style="margin-top: 12px"
          />
        </section>

        <!-- Information -->
        <section v-else-if="currentKey === 'info'">
          <h3 class="section-title">Enter Verification Information</h3>
          <AuthTypeForm
            ref="formRef"
            :meta="meta"
            hide-action
            @change="infoData = $event"
          />
        </section>

        <!-- Document upload -->
        <section v-else-if="currentKey === 'docs'">
          <h3 class="section-title">Upload Documents</h3>
          <DocumentUpload :min-count="1" :max-count="5" @change="documentIds = $event" />
        </section>

        <!-- Video verification -->
        <section v-else-if="currentKey === 'video'">
          <h3 class="section-title">Video Verification (Liveness Detection)</h3>
          <VideoCapture @done="videoResult = $event" />
        </section>

        <!-- Submit -->
        <section v-else>
          <template v-if="isExtreme">
            <el-alert
              title="The Extreme level requires anti-spoofing recording in the OpenForum mobile app"
              description="Complete face liveness + documents + in-app anti-spoofing recording in the app, and the app will submit automatically. The web version does not offer a submission path for the Extreme level."
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
                <el-button type="primary" @click="goCenter">Back to Profile &amp; Verification</el-button>
              </template>
            </el-result>
          </template>

          <template v-else>
            <h3 class="section-title">Review and Submit</h3>
            <ul class="wizard-summary">
              <li>Verification type: {{ meta.label }}</li>
              <li>Verification level: {{ authLevelLabel(level) }}</li>
              <li v-if="showDocs">Documents: {{ documentIds.length }} files</li>
              <li v-if="showVideo">Video verification: {{ videoResult ? (videoResult.liveness.degraded ? 'Recorded (degraded to manual review)' : 'Liveness check complete') : 'Not completed' }}</li>
            </ul>
            <div class="wizard-actions">
              <el-button type="primary" :loading="submitting" @click="submit">Submit Verification</el-button>
            </div>
          </template>
        </section>
      </div>

      <div class="wizard-nav">
        <el-button :disabled="activeIndex === 0" @click="prev">Back</el-button>
        <el-button
          v-if="activeIndex < steps.length - 1"
          type="primary"
          @click="next"
        >
          Next
        </el-button>
        <el-button v-else @click="goCenter">Back to Profile &amp; Verification</el-button>
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
  'Use bright, even lighting; avoid backlight or overexposure',
  'Keep your face unobstructed; remove hats, sunglasses, and masks',
  'Face the camera directly and keep your whole face in frame',
  'Follow the on-screen prompts to complete the random actions in order',
]

const tips = computed(() => levelMeta.value?.tips?.length ? levelMeta.value.tips : DEFAULT_TIPS)

type StepKey = 'intro' | 'info' | 'docs' | 'video' | 'submit'
const steps = computed<{ key: StepKey; title: string }[]>(() => {
  const s: { key: StepKey; title: string }[] = [{ key: 'intro', title: 'Overview' }]
  if (!isExtreme.value) s.push({ key: 'info', title: 'Information' })
  if (showDocs.value) s.push({ key: 'docs', title: 'Documents' })
  if (showVideo.value) s.push({ key: 'video', title: 'Video' })
  s.push({ key: 'submit', title: isExtreme.value ? 'Mobile App Required' : 'Submit' })
  return s
})
const currentKey = computed<StepKey>(() => steps.value[activeIndex.value]?.key ?? 'intro')

function statusText(status: AuthStatus): string {
  return status === 'APPROVED' ? 'Verification approved' : status === 'PENDING' ? 'Submitted, pending review' : 'Verification rejected'
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
    ElMessage.warning('Please upload at least 1 document image')
    return
  }
  if (key === 'video' && !videoResult.value) {
    ElMessage.warning('Please complete video verification first')
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
    ElMessage.success('Verification submitted')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Submission failed'))
  } finally {
    submitting.value = false
  }
}

function goCenter() {
  router.push('/profile')
}

async function load() {
  if (!type.value || !VALID_LEVELS.includes(level.value)) {
    loadError.value = 'Invalid verification type or level'
    return
  }
  try {
    const list = await authenticationApi.types()
    meta.value = (list || []).find((m) => m.type === type.value) || null
    if (!meta.value) loadError.value = 'Verification type not found'
  } catch (e) {
    loadError.value = errorMessage(e, 'Failed to load verification types')
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
