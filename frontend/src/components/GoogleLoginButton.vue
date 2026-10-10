<template>
  <div v-if="enabled" class="google-login">
    <div class="auth-divider"><span>或</span></div>
    <div class="google-btn-wrap">
      <!-- 展示用的自定义按钮，样式跟随站点主题 -->
      <button
        type="button"
        class="google-btn"
        :disabled="!ready"
        tabindex="-1"
        aria-hidden="true"
      >
        <svg class="google-btn-logo" viewBox="0 0 48 48" aria-hidden="true">
          <path
            fill="#EA4335"
            d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"
          />
          <path
            fill="#4285F4"
            d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"
          />
          <path
            fill="#FBBC05"
            d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"
          />
          <path
            fill="#34A853"
            d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"
          />
        </svg>
        <span>{{ label }}</span>
      </button>
      <!-- Google 官方按钮（透明覆盖），仅用于接管点击与回调 -->
      <div ref="slotRef" class="google-slot" :class="{ 'is-ready': ready }"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'

type GoogleButtonText = 'signin_with' | 'signup_with' | 'continue_with'

interface GoogleIdConfig {
  client_id: string
  callback: (response: { credential?: string }) => void
}
interface GoogleButtonOptions {
  type: 'standard'
  theme: 'outline'
  size: 'large'
  shape: 'rectangular'
  text: GoogleButtonText
  logo_alignment: 'left'
  width: number
}
interface GoogleIdentityServices {
  accounts: {
    id: {
      initialize: (config: GoogleIdConfig) => void
      renderButton: (el: HTMLElement, options: GoogleButtonOptions) => void
    }
  }
}

const props = withDefaults(defineProps<{ text?: GoogleButtonText }>(), {
  text: 'continue_with',
})
const emit = defineEmits<{ (e: 'credential', credential: string): void }>()

const LABELS: Record<GoogleButtonText, string> = {
  signin_with: '使用 Google 账号登录',
  signup_with: '使用 Google 账号注册',
  continue_with: '使用 Google 继续',
}
const label = computed(() => LABELS[props.text])

const SCRIPT_SRC = 'https://accounts.google.com/gsi/client'

const clientId = (import.meta.env.VITE_GOOGLE_CLIENT_ID || '').trim()
const enabled = !!clientId

const slotRef = ref<HTMLElement | null>(null)
const ready = ref(false)
let resizeTimer: number | undefined

function gsi(): GoogleIdentityServices | undefined {
  return (window as unknown as { google?: GoogleIdentityServices }).google
}

function loadScript(): Promise<void> {
  return new Promise((resolve, reject) => {
    if (gsi()?.accounts?.id) return resolve()
    const existing = document.querySelector<HTMLScriptElement>(`script[src="${SCRIPT_SRC}"]`)
    if (existing) {
      existing.addEventListener('load', () => resolve(), { once: true })
      existing.addEventListener('error', () => reject(new Error('gsi load failed')), {
        once: true,
      })
      return
    }
    const script = document.createElement('script')
    script.src = SCRIPT_SRC
    script.async = true
    script.defer = true
    script.addEventListener('load', () => resolve(), { once: true })
    script.addEventListener('error', () => reject(new Error('gsi load failed')), { once: true })
    document.head.appendChild(script)
  })
}

function renderButton() {
  const api = gsi()
  const el = slotRef.value
  if (!api?.accounts?.id || !el) return
  const width = Math.min(400, Math.max(200, Math.round(el.clientWidth) || 320))
  el.innerHTML = ''
  api.accounts.id.renderButton(el, {
    type: 'standard',
    // 该按钮被透明覆盖，外观不可见；主题仅作为兜底
    theme: 'outline',
    size: 'large',
    shape: 'rectangular',
    text: props.text,
    logo_alignment: 'left',
    width,
  })
}

function onResize() {
  if (resizeTimer) window.clearTimeout(resizeTimer)
  resizeTimer = window.setTimeout(renderButton, 160)
}

onMounted(async () => {
  if (!enabled) return
  try {
    await loadScript()
    const api = gsi()
    if (!api?.accounts?.id) return
    api.accounts.id.initialize({
      client_id: clientId,
      callback: (response) => {
        if (response?.credential) emit('credential', response.credential)
      },
    })
    await nextTick()
    renderButton()
    ready.value = true
    window.addEventListener('resize', onResize)
  } catch {
    ElMessage.warning('Google 登录加载失败，请使用账号密码登录')
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', onResize)
  if (resizeTimer) window.clearTimeout(resizeTimer)
})
</script>

<style scoped>
.google-login {
  margin-top: 18px;
}
.google-btn-wrap {
  position: relative;
  height: 40px;
}
.google-btn {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  width: 100%;
  height: 100%;
  padding: 0 16px;
  border: none;
  border-radius: var(--of-radius-pill);
  background: var(--of-surface-weak);
  color: var(--of-text);
  font: inherit;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.15s ease;
}
.google-btn:hover:not(:disabled) {
  background: var(--of-border-strong);
}
.google-btn:disabled {
  opacity: 0.6;
  cursor: default;
}
.google-btn-logo {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}
/* Google 官方按钮的透明覆盖层，接管点击 */
.google-slot {
  position: absolute;
  inset: 0;
  overflow: hidden;
  opacity: 0;
  pointer-events: none;
}
.google-slot.is-ready {
  pointer-events: auto;
}
</style>
