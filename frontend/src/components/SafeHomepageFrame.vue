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
import { homepageApi } from '@/api'
import type { HomepageData } from '@/types'

const props = defineProps<{ html: string; userId: number }>()

interface HomepageMessage {
  __openforum_homepage?: number
  type?: string
  height?: number
  path?: string
  url?: string
  nonce?: string
  ok?: boolean
  data?: HomepageData
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

let dataRequested = false
let unmounted = false

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

/** 受信任 shim：拦截导航、上报高度、请求并渲染数据组件。用户 HTML 无法影响它。 */
function shimScript(originValue: string, nonceValue: string): string {
  return `(function () {
  var PARENT_ORIGIN = ${JSON.stringify(originValue)};
  var NONCE = ${JSON.stringify(nonceValue)};
  var EMPTY_TEXT = { authored: '暂无帖子', liked: '暂无点赞', commented: '暂无评论' };
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

  var comps = null;
  function components() {
    if (comps) return comps;
    var nodes = document.querySelectorAll('of-profile, of-stats, of-posts');
    comps = Array.prototype.slice.call(nodes, 0, 20);
    return comps;
  }
  function element(tag, cls, text) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    if (text !== undefined && text !== null) node.textContent = String(text);
    return node;
  }
  function headingOf(comp) {
    var raw = comp.getAttribute('heading');
    if (!raw) return null;
    var text = raw.replace(/^\\s+|\\s+$/g, '');
    if (!text || Array.from(text).length > 30) return null;
    return element('div', 'of-heading', text);
  }
  function validAvatar(value) {
    return typeof value === 'string' && /^\\/forum\\/api\\/media\\/[0-9]+$/.test(value);
  }
  function dateText(value) {
    if (!value) return '';
    var d = new Date(value);
    if (isNaN(d.getTime())) return '';
    var m = d.getMonth() + 1;
    var day = d.getDate();
    return d.getFullYear() + '-' + (m < 10 ? '0' + m : m) + '-' + (day < 10 ? '0' + day : day);
  }
  function roleText(role) {
    return role === 'ADMIN' ? '管理员' : '用户';
  }
  function renderProfile(comp, data) {
    comp.textContent = '';
    var p = data && data.profile;
    if (!p) { comp.textContent = '内容加载失败，请刷新重试'; return; }
    var h = headingOf(comp);
    if (h) comp.appendChild(h);
    var card = element('div', 'of-profile-card');
    if (validAvatar(p.avatar)) {
      var img = document.createElement('img');
      img.className = 'of-avatar';
      img.alt = p.username ? String(p.username) : '';
      img.src = p.avatar;
      card.appendChild(img);
    } else {
      card.appendChild(element('div', 'of-avatar of-avatar-fallback', p.username ? String(p.username).slice(0, 1) : '?'));
    }
    var main = element('div', 'of-profile-main');
    main.appendChild(element('div', 'of-name', p.username || ''));
    main.appendChild(element('div', 'of-meta', roleText(p.role)));
    if (p.bio) main.appendChild(element('div', 'of-bio', p.bio));
    var joined = dateText(p.createdAt);
    if (joined) main.appendChild(element('div', 'of-meta', '加入于 ' + joined));
    card.appendChild(main);
    comp.appendChild(card);
  }
  function renderStats(comp, data) {
    comp.textContent = '';
    var s = data && data.stats;
    if (!s) { comp.textContent = '内容加载失败，请刷新重试'; return; }
    var h = headingOf(comp);
    if (h) comp.appendChild(h);
    var grid = element('div', 'of-stats-grid');
    var items = [['帖子', s.posts], ['评论', s.comments], ['获赞', s.upvotes]];
    for (var i = 0; i < items.length; i++) {
      var box = element('div', 'of-stat');
      var value = Number(items[i][1]);
      box.appendChild(element('div', 'of-stat-value', isFinite(value) ? String(value) : '0'));
      box.appendChild(element('div', 'of-stat-label', items[i][0]));
      grid.appendChild(box);
    }
    comp.appendChild(grid);
  }
  function componentType(comp) {
    var raw = (comp.getAttribute('type') || 'authored').toLowerCase().replace(/^\\s+|\\s+$/g, '');
    return raw === 'liked' || raw === 'commented' ? raw : 'authored';
  }
  function componentLimit(comp) {
    var raw = (comp.getAttribute('limit') || '').replace(/^\\s+|\\s+$/g, '');
    if (!/^[0-9]+$/.test(raw)) return 5;
    var n = parseInt(raw, 10);
    return n >= 1 && n <= 10 ? n : 5;
  }
  function postItem(item) {
    var id = Number(item && item.id);
    if (!isFinite(id) || id <= 0) return null;
    var a = document.createElement('a');
    a.className = 'of-post';
    a.href = '/forum/post/' + Math.floor(id);
    a.appendChild(element('span', 'of-post-title', item.title ? String(item.title) : '（无标题）'));
    var meta = [];
    if (item.forumName) meta.push(String(item.forumName));
    var score = Number(item.score);
    if (isFinite(score)) meta.push(score + ' 赞');
    var comments = Number(item.commentCount);
    if (isFinite(comments)) meta.push(comments + ' 评论');
    var created = dateText(item.createdAt);
    if (created) meta.push(created);
    a.appendChild(element('span', 'of-post-meta', meta.join(' · ')));
    return a;
  }
  function renderPosts(comp, data) {
    comp.textContent = '';
    var type = componentType(comp);
    var list = data ? (type === 'authored' ? data.posts : data[type]) : null;
    if (!Array.isArray(list)) { comp.textContent = '内容加载失败，请刷新重试'; return; }
    var h = headingOf(comp);
    if (h) comp.appendChild(h);
    var limit = componentLimit(comp);
    if (list.length === 0) {
      comp.appendChild(element('div', 'of-empty', EMPTY_TEXT[type]));
      return;
    }
    var wrap = element('div', 'of-post-list');
    var count = 0;
    for (var i = 0; i < list.length && count < limit; i++) {
      var node = postItem(list[i]);
      if (node) { wrap.appendChild(node); count++; }
    }
    if (count === 0) wrap.appendChild(element('div', 'of-empty', EMPTY_TEXT[type]));
    comp.appendChild(wrap);
  }
  function renderAll(data) {
    var list = components();
    for (var i = 0; i < list.length; i++) {
      var comp = list[i];
      var tag = comp.tagName.toLowerCase();
      if (tag === 'of-profile') renderProfile(comp, data);
      else if (tag === 'of-stats') renderStats(comp, data);
      else renderPosts(comp, data);
    }
    reportHeight();
    setTimeout(reportHeight, 50);
  }
  function failAll() {
    var list = components();
    for (var i = 0; i < list.length; i++) list[i].textContent = '内容加载失败，请刷新重试';
    reportHeight();
  }
  var responded = false;
  window.addEventListener('message', function (event) {
    if (event.source !== window.parent) return;
    var data = event.data;
    if (!data || data.__openforum_homepage !== 1) return;
    if (data.type !== 'data-response' || data.nonce !== NONCE || responded) return;
    responded = true;
    if (data.ok && data.data) renderAll(data.data);
    else failAll();
  });

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
  try {
    if (components().length > 0) post({ type: 'data-request', nonce: NONCE });
  } catch (e) {
    /* 保持占位文案 */
  }
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
  of-profile, of-stats, of-posts { display: block; }
  .of-heading { margin: 18px 0 10px; font-size: 16px; font-weight: 600; }
  .of-profile-card { display: flex; gap: 14px; align-items: flex-start; padding: 16px; border: 1px solid #e5e7eb; border-radius: 12px; }
  .of-avatar { width: 64px; height: 64px; border-radius: 50%; object-fit: cover; flex: none; }
  .of-avatar-fallback { display: flex; align-items: center; justify-content: center; background: #eef2ff; color: #4f46e5; font-size: 24px; font-weight: 600; }
  .of-profile-main { min-width: 0; }
  .of-name { font-size: 18px; font-weight: 600; }
  .of-meta { margin-top: 4px; font-size: 12px; color: #6b7280; }
  .of-bio { margin-top: 6px; font-size: 13px; line-height: 1.7; color: #374151; white-space: pre-wrap; }
  .of-stats-grid { display: flex; gap: 12px; }
  .of-stat { flex: 1; padding: 14px; text-align: center; border: 1px solid #e5e7eb; border-radius: 12px; }
  .of-stat-value { font-size: 22px; font-weight: 600; }
  .of-stat-label { margin-top: 4px; font-size: 12px; color: #6b7280; }
  .of-post-list { display: flex; flex-direction: column; gap: 8px; }
  .of-post { display: block; padding: 12px 14px; border: 1px solid #e5e7eb; border-radius: 10px; color: inherit; text-decoration: none; }
  .of-post:hover { border-color: #c7d2fe; background: #f8faff; }
  .of-post-title { display: block; font-size: 14px; font-weight: 600; }
  .of-post-meta { display: block; margin-top: 4px; font-size: 12px; color: #6b7280; }
  .of-empty { padding: 14px; font-size: 13px; color: #9ca3af; text-align: center; border: 1px dashed #e5e7eb; border-radius: 10px; }
  @media (max-width: 560px) {
    .of-stats-grid { flex-direction: column; }
  }
</style>
</head>
<body>
${props.html}
<script nonce="${nonce.value}">${shimScript(origin.value, nonce.value)}<\/script>
</body>
</html>`
})

function respond(
  frame: HTMLIFrameElement | null,
  requestNonce: string,
  ok: boolean,
  data?: HomepageData
) {
  const payload: Record<string, unknown> = {
    __openforum_homepage: 1,
    type: 'data-response',
    nonce: requestNonce,
    ok,
  }
  if (ok && data) payload.data = data
  frame?.contentWindow?.postMessage(payload, '*')
}

async function provideData(requestNonce: string) {
  const frame = frameRef.value
  const userId = Number(props.userId)
  if (!frame?.contentWindow || !Number.isInteger(userId) || userId <= 0) {
    respond(frame, requestNonce, false)
    return
  }
  try {
    const data = await homepageApi.data(userId)
    if (unmounted || requestNonce !== nonce.value) return
    respond(frame, requestNonce, true, data)
  } catch {
    if (unmounted || requestNonce !== nonce.value) return
    respond(frame, requestNonce, false)
  }
}

function onMessage(event: MessageEvent) {
  const frame = frameRef.value
  if (!frame || !frame.contentWindow || event.source !== frame.contentWindow) return
  const data = event.data as HomepageMessage | null
  if (!data || data.__openforum_homepage !== 1) return

  if (data.type === 'data-request') {
    if (data.nonce !== nonce.value || dataRequested) return
    dataRequested = true
    void provideData(nonce.value)
    return
  }

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
    nonce.value = randomNonce()
    dataRequested = false
  }
)

onMounted(() => window.addEventListener('message', onMessage))
onBeforeUnmount(() => {
  unmounted = true
  window.removeEventListener('message', onMessage)
})
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
