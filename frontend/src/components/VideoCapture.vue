<template>
  <div class="video-capture">
    <el-alert
      v-if="errorMsg"
      :title="errorMsg"
      type="error"
      :closable="false"
      show-icon
      style="margin-bottom: 12px"
    />

    <!-- 待开始 -->
    <div v-if="phase === 'idle' || phase === 'error'">
      <p class="video-tip">
        视频认证将随机生成 {{ 3 }} 个动作，请在光线充足、面部无遮挡的环境下正对镜头完成。
      </p>
      <div class="video-actions">
        <el-button type="primary" :disabled="disabled" @click="start">
          开始视频认证
        </el-button>
        <el-button v-if="degradedAvailable" :disabled="disabled" @click="startDegraded">
          仅录制视频，转人工审核
        </el-button>
      </div>
    </div>

    <!-- 进行中 -->
    <div v-else-if="phase === 'initializing'" class="video-loading">
      <span class="video-spinner" />
      正在初始化摄像头与活体模型…
    </div>

    <div v-else class="video-stage">
      <div class="video-frame">
        <video ref="videoEl" class="video-el" playsinline muted />
        <canvas ref="canvasEl" class="video-canvas" />
      </div>

      <div class="video-guide">
        <p class="video-hint">{{ progress?.hint || '请正对镜头' }}</p>
        <ul class="video-action-list">
          <li
            v-for="(action, index) in actions"
            :key="action"
            :class="{
              active: !isDegraded && index === (progress?.currentIndex ?? 0),
              done: !isDegraded && index < (progress?.currentIndex ?? 0),
              off: isDegraded,
            }"
          >
            {{ LIVENESS_ACTION_LABELS[action] }}
          </li>
        </ul>
        <p v-if="isDegraded" class="video-degraded-note">
          降级录制中：请正对镜头并缓慢转头，录制约 6 秒后将提交人工审核。
        </p>

        <div class="video-actions">
          <el-button
            v-if="phase === 'running'"
            :disabled="isDegraded"
            @click="cancel"
          >
            取消
          </el-button>
          <el-button v-else type="primary" :loading="true">上传中…</el-button>
        </div>
      </div>
    </div>

    <!-- 完成 -->
    <div v-if="phase === 'done'" class="video-done">
      <el-alert title="视频已上传" type="success" :closable="false" show-icon />
      <video v-if="previewUrl" :src="previewUrl" class="video-preview" controls />
      <div class="video-actions">
        <el-button @click="reset">重新录制</el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref } from 'vue'
import { authenticationApi } from '@/api'
import type { AuthChallenge } from '@/types'
import { errorMessage } from '@/utils/format'
import {
  LivenessError,
  LIVENESS_ACTION_LABELS,
  isLivenessSupported,
  pickRecorderMime,
  runLiveness,
  type LivenessAction,
  type LivenessPayload,
  type LivenessProgress,
  type VideoAuthResult,
} from '@/utils/liveness'

const props = withDefaults(defineProps<{ disabled?: boolean }>(), { disabled: false })
const emit = defineEmits<{ (e: 'done', result: VideoAuthResult): void }>()

type Phase = 'idle' | 'initializing' | 'running' | 'uploading' | 'done' | 'error'

const phase = ref<Phase>('idle')
const videoEl = ref<HTMLVideoElement | null>(null)
const canvasEl = ref<HTMLCanvasElement | null>(null)
const challenge = ref<AuthChallenge | null>(null)
const progress = ref<LivenessProgress | null>(null)
const errorMsg = ref('')
const degradedAvailable = ref(false)
const result = ref<VideoAuthResult | null>(null)
const previewUrl = ref('')

let controller: AbortController | null = null

const isDegraded = computed(() => !!result.value?.liveness.degraded)
const actions = computed<LivenessAction[]>(
  () => (challenge.value?.actions as LivenessAction[]) || []
)

/** 供父组件读取已完成的结果 */
function getResult(): VideoAuthResult | null {
  return result.value
}

defineExpose({ getResult })

function reset() {
  result.value = null
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value)
  previewUrl.value = ''
  progress.value = null
  phase.value = 'idle'
}

async function start() {
  if (props.disabled) return
  if (!isLivenessSupported()) {
    errorMsg.value = '当前浏览器不支持摄像头活体检测，可尝试降级录制。'
    degradedAvailable.value = true
    phase.value = 'error'
    return
  }
  errorMsg.value = ''
  degradedAvailable.value = false
  phase.value = 'initializing'
  controller = new AbortController()
  try {
    const ch = await authenticationApi.challenge()
    challenge.value = ch
    phase.value = 'running'
    await nextTick()
    const { blob, liveness } = await runLiveness({
      videoEl: videoEl.value!,
      canvasEl: canvasEl.value!,
      actions: ch.actions as LivenessAction[],
      nonce: ch.nonce,
      challengeId: ch.challengeId,
      signal: controller.signal,
      onProgress: (p) => {
        progress.value = p
      },
    })
    await upload(blob, liveness)
  } catch (e) {
    handleError(e)
  }
}

async function startDegraded() {
  if (props.disabled) return
  errorMsg.value = ''
  phase.value = 'initializing'
  try {
    const ch = challenge.value ?? (await authenticationApi.challenge())
    challenge.value = ch
    phase.value = 'running'
    await nextTick()
    const { blob, startedAt, endedAt } = await recordPlainVideo()
    const liveness: LivenessPayload = {
      engine: 'none',
      challengeId: ch.challengeId,
      faceCount: 0,
      actions: [],
      passed: false,
      degraded: true,
      startedAt,
      endedAt,
    }
    await upload(blob, liveness)
  } catch (e) {
    handleError(e)
  }
}

async function upload(blob: Blob, liveness: LivenessPayload) {
  phase.value = 'uploading'
  const file = new File([blob], `liveness-${Date.now()}.webm`, {
    type: blob.type || 'video/webm',
  })
  const res = await authenticationApi.uploadMedia(file, 'VIDEO')
  const payload: VideoAuthResult = {
    videoMediaId: res.mediaId,
    challengeId: challenge.value?.challengeId || '',
    liveness,
  }
  result.value = payload
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value)
  previewUrl.value = URL.createObjectURL(blob)
  phase.value = 'done'
  emit('done', payload)
}

function cancel() {
  controller?.abort()
  phase.value = 'idle'
}

function handleError(e: unknown) {
  if (e instanceof LivenessError) {
    if (e.code === 'CANCELLED') {
      phase.value = 'idle'
      return
    }
    if (e.code === 'MODEL_LOAD_FAILED' || e.code === 'UNSUPPORTED') {
      degradedAvailable.value = true
    }
    errorMsg.value = e.message
  } else {
    errorMsg.value = errorMessage(e, '视频认证失败，请重试')
  }
  phase.value = 'error'
}

/** 降级：仅录制视频（仍叠加 nonce 水印），不做活体判定 */
async function recordPlainVideo(): Promise<{ blob: Blob; startedAt: string; endedAt: string }> {
  const video = videoEl.value!
  const canvas = canvasEl.value!
  const stream = await navigator.mediaDevices.getUserMedia({
    video: { facingMode: 'user', width: { ideal: 640 }, height: { ideal: 480 } },
    audio: false,
  })
  video.srcObject = stream
  video.muted = true
  await video.play()

  const ctx = canvas.getContext('2d')!
  const mime = pickRecorderMime()
  if (!mime) {
    stream.getTracks().forEach((t) => t.stop())
    throw new LivenessError('UNSUPPORTED', '当前浏览器不支持视频录制')
  }
  const recorder = new MediaRecorder(canvas.captureStream(25), { mimeType: mime })
  const chunks: BlobPart[] = []
  recorder.ondataavailable = (ev) => {
    if (ev.data && ev.data.size > 0) chunks.push(ev.data)
  }
  recorder.start(200)

  const startedAt = new Date().toISOString()
  const start = performance.now()
  const duration = 6000
  const nonce = challenge.value?.nonce || ''

  await new Promise<void>((resolve) => {
    const draw = () => {
      const elapsed = performance.now() - start
      const w = video.videoWidth || 640
      const h = video.videoHeight || 480
      if (canvas.width !== w || canvas.height !== h) {
        canvas.width = w
        canvas.height = h
      }
      if (video.readyState >= 2) {
        ctx.save()
        ctx.translate(w, 0)
        ctx.scale(-1, 1)
        ctx.drawImage(video, 0, 0, w, h)
        ctx.restore()
        const remain = Math.max(0, Math.ceil((duration - elapsed) / 1000))
        ctx.save()
        ctx.font = `${Math.max(14, Math.round(w / 32))}px monospace`
        ctx.textBaseline = 'bottom'
        ctx.fillStyle = 'rgba(0,0,0,0.45)'
        ctx.fillRect(0, h - Math.round(h / 12), w, Math.round(h / 12))
        ctx.fillStyle = '#fff'
        ctx.fillText(`${nonce}  剩余 ${remain}s`, 12, h - 10)
        ctx.restore()
      }
      if (elapsed >= duration) {
        resolve()
        return
      }
      requestAnimationFrame(draw)
    }
    requestAnimationFrame(draw)
  })

  await new Promise<void>((resolve) => {
    recorder.onstop = () => resolve()
    recorder.stop()
  })
  stream.getTracks().forEach((t) => t.stop())
  video.srcObject = null
  const endedAt = new Date().toISOString()
  const blob = new Blob(chunks, { type: mime })
  if (!blob.size) throw new LivenessError('UNSUPPORTED', '视频录制失败，请重试')
  return { blob, startedAt, endedAt }
}

onBeforeUnmount(() => {
  controller?.abort()
  if (previewUrl.value) URL.revokeObjectURL(previewUrl.value)
})
</script>

<style scoped>
.video-tip {
  color: var(--of-text-muted);
  font-size: 13px;
  margin-bottom: 12px;
}
.video-loading {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--of-text-muted);
  padding: 20px 0;
}
.video-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--el-border-color, #dcdfe6);
  border-top-color: var(--el-color-primary);
  border-radius: 50%;
  animation: video-spin 0.8s linear infinite;
}
@keyframes video-spin {
  to {
    transform: rotate(360deg);
  }
}
.video-stage {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.video-frame {
  position: relative;
  width: 100%;
  max-width: 480px;
  aspect-ratio: 4 / 3;
  background: #000;
  border-radius: 8px;
  overflow: hidden;
}
.video-el {
  display: none;
}
.video-canvas {
  width: 100%;
  height: 100%;
  display: block;
}
.video-hint {
  font-weight: 600;
  margin-bottom: 8px;
}
.video-action-list {
  list-style: none;
  padding: 0;
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.video-action-list li {
  padding: 4px 12px;
  border-radius: 14px;
  border: 1px solid var(--el-border-color, #dcdfe6);
  font-size: 13px;
  color: var(--of-text-muted);
}
.video-action-list li.active {
  border-color: var(--el-color-primary);
  color: var(--el-color-primary);
  font-weight: 600;
}
.video-action-list li.done {
  border-color: var(--el-color-success);
  color: var(--el-color-success);
}
.video-degraded-note {
  color: var(--el-color-warning);
  font-size: 13px;
}
.video-actions {
  margin-top: 12px;
  display: flex;
  gap: 10px;
}
.video-done {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.video-preview {
  width: 100%;
  max-width: 320px;
  border-radius: 8px;
  background: #000;
}
</style>
