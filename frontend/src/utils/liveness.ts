import { FaceLandmarker, FilesetResolver, type FaceLandmarkerResult } from '@mediapipe/tasks-vision'

/** 随机动作池（与后端 AuthChallengeService 保持一致） */
export type LivenessAction = 'BLINK' | 'OPEN_MOUTH' | 'TURN_LEFT' | 'TURN_RIGHT' | 'NOD'

export const LIVENESS_ACTION_LABELS: Record<LivenessAction, string> = {
  BLINK: '眨眨眼',
  OPEN_MOUTH: '张张嘴',
  TURN_LEFT: '向左转头',
  TURN_RIGHT: '向右转头',
  NOD: '点点头',
}

export interface LivenessActionRecord {
  action: LivenessAction
  passed: boolean
  atMs: number
}

export interface LivenessPayload {
  engine: string
  challengeId: string
  faceCount: number
  actions: LivenessActionRecord[]
  passed: boolean
  degraded?: boolean
  startedAt: string
  endedAt: string
}

export interface LivenessProgress {
  currentIndex: number
  currentAction: LivenessAction | null
  actions: LivenessActionRecord[]
  faceCount: number
  hint: string
  /** 0-1，整体进度 */
  ratio: number
}

export type LivenessErrorCode =
  | 'UNSUPPORTED'
  | 'PERMISSION_DENIED'
  | 'NO_CAMERA'
  | 'MODEL_LOAD_FAILED'
  | 'CANCELLED'

export class LivenessError extends Error {
  code: LivenessErrorCode
  constructor(code: LivenessErrorCode, message: string) {
    super(message)
    this.name = 'LivenessError'
    this.code = code
  }
}

export interface RunLivenessOptions {
  videoEl: HTMLVideoElement
  canvasEl: HTMLCanvasElement
  actions: LivenessAction[]
  nonce: string
  challengeId?: string
  /** 单个动作的最长等待时间（毫秒），默认 20s */
  actionTimeoutMs?: number
  signal?: AbortSignal
  onProgress?: (state: LivenessProgress) => void
}

export interface RunLivenessResult {
  blob: Blob
  liveness: LivenessPayload
}

/** 视频认证步骤完成后回传给向导的数据 */
export interface VideoAuthResult {
  videoMediaId: number
  challengeId: string
  liveness: LivenessPayload
}

const WASM_DIR = `${import.meta.env.BASE_URL || '/'}mediapipe`
const MODEL_URL = `${import.meta.env.BASE_URL || '/'}models/face_landmarker.task`

let landmarkerPromise: Promise<FaceLandmarker> | null = null

/** 检测浏览器是否具备活体检测能力 */
export function isLivenessSupported(): boolean {
  return (
    typeof navigator !== 'undefined' &&
    !!navigator.mediaDevices?.getUserMedia &&
    typeof window !== 'undefined' &&
    typeof window.MediaRecorder !== 'undefined' &&
    typeof HTMLCanvasElement !== 'undefined' &&
    typeof HTMLCanvasElement.prototype.captureStream === 'function'
  )
}

async function createLandmarker(): Promise<FaceLandmarker> {
  const fileset = await FilesetResolver.forVisionTasks(WASM_DIR)
  const options = {
    baseOptions: { modelAssetPath: MODEL_URL, delegate: 'GPU' as const },
    runningMode: 'VIDEO' as const,
    numFaces: 2,
    outputFaceBlendshapes: true,
    outputFacialTransformationMatrixes: true,
  }
  try {
    return await FaceLandmarker.createFromOptions(fileset, options)
  } catch {
    // GPU 不可用时回退到 CPU
    return await FaceLandmarker.createFromOptions(fileset, {
      ...options,
      baseOptions: { modelAssetPath: MODEL_URL, delegate: 'CPU' as const },
    })
  }
}

/** 惰性加载并缓存 FaceLandmarker */
async function getFaceLandmarker(): Promise<FaceLandmarker> {
  if (!landmarkerPromise) {
    landmarkerPromise = createLandmarker().catch((e) => {
      landmarkerPromise = null
      throw new LivenessError('MODEL_LOAD_FAILED', `活体模型加载失败：${(e as Error)?.message || e}`)
    })
  }
  return landmarkerPromise
}

function blendshapeScore(result: FaceLandmarkerResult, name: string): number {
  const shapes = result.faceBlendshapes
  if (!shapes || !shapes.length) return 0
  const found = shapes[0].categories.find((c) => c.categoryName === name)
  return found ? found.score : 0
}

interface HeadPose {
  /** 偏航比例：0.5 附近为正向，越大代表鼻子越靠图像右侧 */
  yaw: number
  /** 俯仰比例：越大代表鼻子相对眼线越靠下 */
  pitch: number
}

/** 由面部关键点估计头部姿态比例 */
function estimateHeadPose(result: FaceLandmarkerResult): HeadPose | null {
  const lm = result.faceLandmarks?.[0]
  if (!lm || lm.length < 468) return null
  const nose = lm[1]
  const leftEdge = lm[234]
  const rightEdge = lm[454]
  const width = rightEdge.x - leftEdge.x
  const yaw = Math.abs(width) > 1e-6 ? (nose.x - leftEdge.x) / width : 0.5
  const forehead = lm[10]
  const chin = lm[152]
  const eyeMidY = (lm[33].y + lm[263].y) / 2
  const faceH = chin.y - forehead.y
  const pitch = Math.abs(faceH) > 1e-6 ? (nose.y - eyeMidY) / faceH : 0.5
  return { yaw, pitch }
}

/**
 * 在 videoEl 上执行活体检测与动作序列，同时录制（含 nonce 水印）。
 * 全过程在浏览器端完成，返回 webm Blob 与 liveness 载荷。
 */
export async function runLiveness(options: RunLivenessOptions): Promise<RunLivenessResult> {
  const { videoEl, canvasEl, actions, nonce, challengeId, onProgress, signal } = options
  const actionTimeoutMs = options.actionTimeoutMs ?? 20000

  if (!isLivenessSupported()) {
    throw new LivenessError('UNSUPPORTED', '当前浏览器不支持摄像头活体检测')
  }
  if (!actions.length) {
    throw new LivenessError('UNSUPPORTED', '未提供动作序列')
  }

  let stream: MediaStream | null = null
  let landmarker: FaceLandmarker
  try {
    landmarker = await getFaceLandmarker()
  } catch (e) {
    throw e instanceof LivenessError ? e : new LivenessError('MODEL_LOAD_FAILED', '活体模型加载失败')
  }

  try {
    stream = await navigator.mediaDevices.getUserMedia({
      video: { facingMode: 'user', width: { ideal: 640 }, height: { ideal: 480 } },
      audio: false,
    })
  } catch (e) {
    const name = (e as DOMException)?.name
    if (name === 'NotAllowedError' || name === 'SecurityError') {
      throw new LivenessError('PERMISSION_DENIED', '摄像头权限被拒绝，请允许后重试')
    }
    throw new LivenessError('NO_CAMERA', '未检测到可用摄像头')
  }

  videoEl.srcObject = stream
  videoEl.muted = true
  videoEl.playsInline = true
  try {
    await videoEl.play()
  } catch {
    /* 部分浏览器需用户手势后才可播放，忽略 */
  }

  const ctx = canvasEl.getContext('2d')
  const records: LivenessActionRecord[] = actions.map((a) => ({ action: a, passed: false, atMs: 0 }))
  const startedAt = new Date().toISOString()
  const startTs = performance.now()

  let faceCount = 0
  let currentIndex = 0
  let lastProgress = 0
  let lastBlinkClosed = false
  let mouthFrames = 0
  // 头部姿态基线（默认 0.5），用前若干帧校准
  let baseYaw = 0.5
  let basePitch = 0.5
  let calibFrames = 0
  let calibSumYaw = 0
  let calibSumPitch = 0
  let calibDone = false
  // 当前动作的“已越过阈值”标记（用于回正判定）
  let crossed = false
  let actionStartTs = startTs
  let done = false

  let recorder: MediaRecorder | null = null
  const chunks: BlobPart[] = []
  const mime = pickRecorderMime()
  if (mime) {
    try {
      recorder = new MediaRecorder(canvasEl.captureStream(25), { mimeType: mime })
      recorder.ondataavailable = (ev) => {
        if (ev.data && ev.data.size > 0) chunks.push(ev.data)
      }
      recorder.start(200)
    } catch {
      recorder = null
    }
  }

  const emit = () => {
    if (!onProgress) return
    onProgress({
      currentIndex,
      currentAction: actions[currentIndex] ?? null,
      actions: [...records],
      faceCount,
      hint: buildHint(currentIndex, records, faceCount, actions),
      ratio: records.filter((r) => r.passed).length / actions.length,
    })
  }

  const finishAction = (atMs: number) => {
    const rec = records[currentIndex]
    if (!rec) return
    rec.passed = true
    rec.atMs = atMs
    crossed = false
    mouthFrames = 0
    currentIndex += 1
    actionStartTs = performance.now()
    emit()
  }

  return await new Promise<RunLivenessResult>((resolve, reject) => {
    const cleanup = () => {
      cancelAnimationFrame(rafId)
      if (recorder && recorder.state !== 'inactive') {
        try {
          recorder.stop()
        } catch {
          /* ignore */
        }
      }
      stream?.getTracks().forEach((t) => t.stop())
      videoEl.srcObject = null
    }

    const abortCheck = () => {
      if (signal?.aborted) {
        throw new LivenessError('CANCELLED', '已取消')
      }
    }

    const loop = () => {
      if (done) return
      try {
        abortCheck()
      } catch (e) {
        cleanup()
        reject(e)
        return
      }

      const now = performance.now()
      const w = videoEl.videoWidth || 640
      const h = videoEl.videoHeight || 480
      if (canvasEl.width !== w || canvasEl.height !== h) {
        canvasEl.width = w
        canvasEl.height = h
      }

      // 镜像绘制 + nonce 水印
      if (ctx && videoEl.readyState >= 2) {
        ctx.save()
        ctx.translate(w, 0)
        ctx.scale(-1, 1)
        ctx.drawImage(videoEl, 0, 0, w, h)
        ctx.restore()
        drawWatermark(ctx, w, h, nonce, now - startTs)
      }

      if (videoEl.readyState >= 2 && currentIndex < actions.length) {
        let result: FaceLandmarkerResult | null = null
        try {
          result = landmarker.detectForVideo(videoEl, now)
        } catch {
          result = null
        }

        if (result) {
          faceCount = result.faceLandmarks?.length ?? 0
          if (faceCount === 1) {
            // 基线校准
            const pose = estimateHeadPose(result)
            if (pose && !calibDone) {
              calibSumYaw += pose.yaw
              calibSumPitch += pose.pitch
              calibFrames += 1
              if (calibFrames >= 20) {
                baseYaw = calibSumYaw / calibFrames
                basePitch = calibSumPitch / calibFrames
                calibDone = true
              }
            }
            if (calibDone) {
              const action = actions[currentIndex]
              const passed = evaluateAction(action, {
                result,
                pose,
                baseYaw,
                basePitch,
                crossed,
                mouthFrames,
                lastBlinkClosed,
              })
              crossed = passed.crossed
              mouthFrames = passed.mouthFrames
              lastBlinkClosed = passed.blinkClosed
              if (passed.done) {
                finishAction(Math.round(now - startTs))
                if (currentIndex >= actions.length) {
                  done = true
                  window.setTimeout(() => finalize(), 400)
                  return
                }
              }
            }
          } else {
            // 人脸数量异常时重置校准与当前动作状态
            crossed = false
            mouthFrames = 0
          }

          if (now - lastProgress > 120) {
            lastProgress = now
            emit()
          }

          if (now - actionStartTs > actionTimeoutMs) {
            actionStartTs = now
            emit()
          }
        }
      }

      rafId = requestAnimationFrame(loop)
    }

    const finalize = () => {
      cleanup()
      const endedAt = new Date().toISOString()
      const liveness: LivenessPayload = {
        engine: 'mediapipe-face-landmarker',
        challengeId: challengeId || '',
        faceCount: 1,
        actions: records,
        passed: records.every((r) => r.passed),
        startedAt,
        endedAt,
      }
      const blob = new Blob(chunks, { type: mime || 'video/webm' })
      if (!blob.size) {
        reject(new LivenessError('UNSUPPORTED', '视频录制失败，请重试或使用降级方式'))
        return
      }
      resolve({ blob, liveness })
    }

    let rafId = requestAnimationFrame(loop)
    // 允许外部取消
    signal?.addEventListener('abort', () => {
      if (done) return
      done = true
      cleanup()
      reject(new LivenessError('CANCELLED', '已取消'))
    })
  })
}

export function pickRecorderMime(): string | null {
  const candidates = ['video/webm;codecs=vp8', 'video/webm;codecs=vp9', 'video/webm']
  for (const c of candidates) {
    if (typeof MediaRecorder !== 'undefined' && MediaRecorder.isTypeSupported?.(c)) return c
  }
  return null
}

function drawWatermark(
  ctx: CanvasRenderingContext2D,
  w: number,
  h: number,
  nonce: string,
  elapsedMs: number
) {
  const seconds = (elapsedMs / 1000).toFixed(1)
  ctx.save()
  ctx.font = `${Math.max(14, Math.round(w / 32))}px monospace`
  ctx.textBaseline = 'bottom'
  ctx.fillStyle = 'rgba(0,0,0,0.45)'
  ctx.fillRect(0, h - Math.round(h / 12), w, Math.round(h / 12))
  ctx.fillStyle = '#fff'
  ctx.fillText(`${nonce}  ${seconds}s`, 12, h - 10)
  ctx.restore()
}

interface ActionEvalContext {
  result: FaceLandmarkerResult
  pose: HeadPose | null
  baseYaw: number
  basePitch: number
  crossed: boolean
  mouthFrames: number
  lastBlinkClosed: boolean
}

interface ActionEvalOutcome {
  done: boolean
  crossed: boolean
  mouthFrames: number
  blinkClosed: boolean
}

function evaluateAction(
  action: LivenessAction,
  ctx: ActionEvalContext
): ActionEvalOutcome {
  const { result, pose, baseYaw, basePitch } = ctx
  const out: ActionEvalOutcome = {
    done: false,
    crossed: ctx.crossed,
    mouthFrames: ctx.mouthFrames,
    blinkClosed: ctx.lastBlinkClosed,
  }

  switch (action) {
    case 'BLINK': {
      const l = blendshapeScore(result, 'eyeBlinkLeft')
      const r = blendshapeScore(result, 'eyeBlinkRight')
      const closed = l > 0.5 && r > 0.5
      const open = l < 0.35 && r < 0.35
      if (closed) out.blinkClosed = true
      if (out.blinkClosed && open) {
        out.done = true
        out.blinkClosed = false
      }
      break
    }
    case 'OPEN_MOUTH': {
      const jaw = blendshapeScore(result, 'jawOpen')
      if (jaw > 0.5) {
        out.mouthFrames = ctx.mouthFrames + 1
        if (out.mouthFrames >= 3) out.done = true
      } else {
        out.mouthFrames = 0
      }
      break
    }
    case 'TURN_LEFT':
    case 'TURN_RIGHT': {
      if (!pose) break
      const delta = pose.yaw - baseYaw
      const left = action === 'TURN_LEFT'
      const reached = left ? delta > 0.1 : delta < -0.1
      const backToNeutral = Math.abs(delta) < 0.045
      if (reached) out.crossed = true
      if (out.crossed && backToNeutral) out.done = true
      break
    }
    case 'NOD': {
      if (!pose) break
      const delta = pose.pitch - basePitch
      const reached = delta > 0.09
      const backToNeutral = Math.abs(delta) < 0.04
      if (reached) out.crossed = true
      if (out.crossed && backToNeutral) out.done = true
      break
    }
  }
  return out
}

function buildHint(
  index: number,
  records: LivenessActionRecord[],
  faceCount: number,
  actions: LivenessAction[]
): string {
  if (faceCount === 0) return '未检测到人脸，请正对摄像头并保持光线充足'
  if (faceCount > 1) return '检测到多张人脸，请确保画面中只有你一人'
  const action = actions[index]
  if (!action) return '检测完成'
  const label = LIVENESS_ACTION_LABELS[action]
  return `请按提示完成：${label}（已完成 ${records.filter((r) => r.passed).length}/${actions.length}）`
}
