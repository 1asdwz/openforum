<template>
  <div v-if="enabled" class="google-login">
    <div class="auth-divider"><span>或</span></div>
    <div ref="slotRef" class="google-slot"></div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useUiStore } from '@/stores/ui'

type GoogleButtonText = 'signin_with' | 'signup_with' | 'continue_with'

interface GoogleIdConfig {
  client_id: string
  callback: (response: { credential?: string }) => void
}
interface GoogleButtonOptions {
  type: 'standard'
  theme: 'outline' | 'filled_black'
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

const SCRIPT_SRC = 'https://accounts.google.com/gsi/client'

const ui = useUiStore()
const clientId = (import.meta.env.VITE_GOOGLE_CLIENT_ID || '').trim()
const enabled = !!clientId

const slotRef = ref<HTMLElement | null>(null)
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
  const width = Math.min(400, Math.max(200, el.clientWidth || 320))
  el.innerHTML = ''
  api.accounts.id.renderButton(el, {
    type: 'standard',
    theme: ui.isDark ? 'filled_black' : 'outline',
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
    window.addEventListener('resize', onResize)
  } catch {
    ElMessage.warning('Google 登录加载失败，请使用账号密码登录')
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', onResize)
  if (resizeTimer) window.clearTimeout(resizeTimer)
})

// 明暗主题切换时用对应主题重绘按钮（dark 下用 filled_black）
if (enabled) {
  watch(
    () => ui.isDark,
    () => renderButton(),
  )
}
</script>

<style scoped>
.google-login {
  margin-top: 18px;
}
.google-slot {
  display: flex;
  justify-content: center;
  min-height: 40px;
}
</style>
