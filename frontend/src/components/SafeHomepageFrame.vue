<template>
  <div class="safe-homepage">
    <div v-if="!envOk" class="safe-homepage-error">
      当前环境不支持安全预览（缺少有效的站点来源），请通过 https 访问本站。
    </div>
    <iframe
      v-else
      ref="frameRef"
      class="safe-homepage-frame"
      sandbox="allow-scripts"
      referrerpolicy="no-referrer"
      scrolling="no"
      :srcdoc="documentHtml"
      :style="{ height: `${height}px` }"
      title="用户自定义主页"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessageBox } from 'element-plus'

const props = defineProps<{ html: string }>()

interface HomepageMessage {
  __openforum_homepage?: number
  type?: string
  height?: number
  path?: string
  url?: string
}

const MIN_HEIGHT = 480
const MAX_HEIGHT = 4000

const router = useRouter()
const frameRef = ref<HTMLIFrameElement | null>(null)
const height = ref(MIN_HEIGHT)

/** 只接受 http(s) 来源；file:// 等环境直接拒绝加载。 */
const nonce = ref(randomNonce())
const origin = computed(() => {
  const o = window.location.origin
  return /^https?:\/\//i.test(o) ? o : ''
})
const envOk = computed(() => !!origin.value && !!nonce.value)

function randomNonce(): string {
  try {
    if (!window.crypto?.getRandomValues) return ''
    const bytes = new Uint8Array(16)
    window.crypto.getRandomValues(bytes)
    return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
  } catch {
    return ''
  }
}

/** 受信任 shim：拦截外链/站内导航、上报高度。用户 HTML 无法影响它。 */
function shimScript(originValue: string): string {
  return `(function () {
  var PARENT_ORIGIN = ${JSON.stringify(originValue)};
  function post(payload) {
    payload.__openforum_homepage = 1;
    parent.postMessage(payload, PARENT_ORIGIN);
  }
  function closestAnchor(node) {
    while (node && node !== document) {
      if (node.tagName === 'A' && node.getAttribute('href')) return node;
      node = node.parentNode;
    }
    return null;
  }
  function handle(event) {
    if (event.defaultPrevented) return;
    var a = closestAnchor(event.target);
    if (!a) return;
    var href = (a.getAttribute('href') || '').trim();
    if (!href || href.charAt(0) === '#') return;
    if (href.indexOf('mailto:') === 0) return;
    event.preventDefault();
    if (href.charAt(0) === '/') {
      if (href.charAt(1) === '/') return;
      var path = (href === '/forum' || href.indexOf('/forum/') === 0) ? href.slice(6) : href;
      post({ type: 'internal', path: path || '/' });
      return;
    }
    if (/^https?:/i.test(href)) {
      var host = '';
      try { host = new URL(href).host; } catch (e) { host = href; }
      post({ type: 'external', url: href, host: host });
    }
  }
  document.addEventListener('click', handle, true);
  document.addEventListener('auxclick', handle, true);
  document.addEventListener('submit', function (event) { event.preventDefault(); }, true);
  function reportHeight() {
    var doc = document.documentElement;
    var body = document.body;
    var h = Math.max(doc ? doc.scrollHeight : 0, body ? body.scrollHeight : 0);
    post({ type: 'height', height: h });
  }
  if (window.ResizeObserver) {
    var ro = new ResizeObserver(reportHeight);
    ro.observe(document.documentElement);
    if (document.body) ro.observe(document.body);
  }
  window.addEventListener('load', reportHeight);
  reportHeight();
  setTimeout(reportHeight, 100);
})();`
}

const documentHtml = computed(() => {
  if (!envOk.value) return ''
  const csp = [
    "default-src 'none'",
    `img-src ${origin.value}/forum/api/media/`,
    "style-src 'unsafe-inline'",
    `script-src 'nonce-${nonce.value}'`,
    "connect-src 'none'",
    "font-src 'none'",
    "media-src 'none'",
    "object-src 'none'",
    "frame-src 'none'",
    "worker-src 'none'",
    "form-action 'none'",
    "base-uri 'none'",
  ].join('; ')
  return `<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="${csp}">
<style>
  html, body { margin: 0; padding: 0; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "PingFang SC", sans-serif;
    color: #1f2328;
    background: transparent;
    overflow-wrap: anywhere;
  }
  img { max-width: 100%; height: auto; }
</style>
</head>
<body>
${props.html}
<script nonce="${nonce.value}">${shimScript(origin.value)}<\/script>
</body>
</html>`
})

function onMessage(event: MessageEvent) {
  const frame = frameRef.value
  if (!frame || !frame.contentWindow || event.source !== frame.contentWindow) return
  const data = event.data as HomepageMessage | null
  if (!data || data.__openforum_homepage !== 1) return

  if (data.type === 'height') {
    const raw = Number(data.height)
    if (!Number.isFinite(raw)) return
    height.value = Math.min(MAX_HEIGHT, Math.max(MIN_HEIGHT, Math.round(raw)))
    return
  }

  if (data.type === 'internal') {
    const path = typeof data.path === 'string' ? data.path : ''
    if (!path.startsWith('/') || path.startsWith('//')) return
    router.push(path)
    return
  }

  if (data.type === 'external') {
    const url = typeof data.url === 'string' ? data.url : ''
    if (!/^https?:\/\//i.test(url)) return
    let host = ''
    try {
      host = new URL(url).host
    } catch {
      return
    }
    ElMessageBox.confirm(
      `即将离开本站，前往外部网站：${host}\n外部网站可能收集你的信息，请注意账号与财产安全。`,
      '安全风险提示',
      {
        confirmButtonText: '继续访问',
        cancelButtonText: '留在本站',
        type: 'warning',
      }
    )
      .then(() => {
        window.open(url, '_blank', 'noopener,noreferrer')
      })
      .catch(() => {
        /* 用户取消 */
      })
  }
}

watch(
  () => props.html,
  () => {
    height.value = MIN_HEIGHT
  }
)

onMounted(() => window.addEventListener('message', onMessage))
onBeforeUnmount(() => window.removeEventListener('message', onMessage))
</script>

<style scoped>
.safe-homepage {
  width: 100%;
}
.safe-homepage-frame {
  display: block;
  width: 100%;
  min-height: 480px;
  border: 0;
  background: transparent;
}
.safe-homepage-error {
  padding: 24px;
  border: 1px dashed var(--of-border);
  border-radius: var(--of-radius);
  color: var(--of-text-muted);
  text-align: center;
}
</style>
