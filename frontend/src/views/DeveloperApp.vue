<template>
  <div class="page-grid no-right">
    <LeftRail />

    <div class="feed-column">
      <router-link to="/developers" class="dev-back">← All applications</router-link>

      <el-skeleton v-if="loading" :rows="8" animated />

      <div v-else-if="error" class="state-box error">
        {{ error }}
        <div style="margin-top: 12px">
          <el-button size="small" @click="load">Retry</el-button>
        </div>
      </div>

      <template v-else-if="app">
        <header v-reveal class="dev-detail-head">
          <span class="of-eyebrow">
            {{ app.clientType === 'PUBLIC' ? 'Public client' : 'Confidential client' }}
          </span>
          <h1 class="page-title">{{ app.name }}</h1>
          <p class="page-desc">
            {{ app.description || 'No description yet — add one below so users know what they are authorizing.' }}
          </p>
        </header>

        <section v-reveal="40" class="content-card dev-card">
          <div class="dev-card-head">
            <h2>Credentials</h2>
            <span class="dev-badge" :class="app.status === 'ACTIVE' ? 'ok' : 'off'">
              {{ app.status === 'ACTIVE' ? 'Active' : 'Disabled' }}
            </span>
          </div>

          <div class="dev-kv">
            <span class="dev-kv-key">Client ID</span>
            <code class="dev-kv-value">{{ app.clientId }}</code>
            <el-button size="small" text @click="copy(app.clientId)">Copy</el-button>
          </div>

          <div class="dev-kv">
            <span class="dev-kv-key">Client secret</span>
            <code class="dev-kv-value">{{ app.hasSecret ? '••••••••••••••••••••••••' : 'Not used (PKCE)' }}</code>
            <el-button
              v-if="app.hasSecret"
              size="small"
              text
              :loading="resetting"
              @click="resetSecret"
            >
              Reset
            </el-button>
          </div>

          <div class="dev-kv">
            <span class="dev-kv-key">Auth method</span>
            <span class="dev-kv-value plain">
              {{ app.clientType === 'PUBLIC' ? 'Authorization code + PKCE (S256)' : 'Authorization code + client secret' }}
            </span>
            <span />
          </div>

          <p class="dev-hint">
            The secret is stored hashed. Resetting it immediately invalidates the previous secret.
          </p>
        </section>

        <section v-reveal="80" class="content-card dev-card">
          <div class="dev-card-head">
            <h2>OAuth endpoints</h2>
            <el-button size="small" text @click="copyEndpoints">Copy all</el-button>
          </div>

          <div class="dev-kv">
            <span class="dev-kv-key">Authorize</span>
            <code class="dev-kv-value">{{ endpoints.authorize }}</code>
            <el-button size="small" text @click="copy(endpoints.authorize)">Copy</el-button>
          </div>
          <div class="dev-kv">
            <span class="dev-kv-key">Token</span>
            <code class="dev-kv-value">{{ endpoints.token }}</code>
            <el-button size="small" text @click="copy(endpoints.token)">Copy</el-button>
          </div>
          <div class="dev-kv">
            <span class="dev-kv-key">Userinfo</span>
            <code class="dev-kv-value">{{ endpoints.userinfo }}</code>
            <el-button size="small" text @click="copy(endpoints.userinfo)">Copy</el-button>
          </div>

          <div class="dev-redirects">
            <span class="dev-kv-key">Registered redirect URIs</span>
            <ul>
              <li v-for="uri in app.redirectUris" :key="uri">
                <code>{{ uri }}</code>
              </li>
            </ul>
          </div>

          <details class="dev-snippet">
            <summary>Example: exchange an authorization code</summary>
            <pre><code>{{ tokenExample }}</code></pre>
          </details>
        </section>

        <section v-reveal="120" class="content-card dev-card">
          <div class="dev-card-head">
            <h2>Settings</h2>
            <el-switch
              v-model="form.active"
              inline-prompt
              active-text="On"
              inactive-text="Off"
            />
          </div>

          <el-form label-position="top">
            <el-form-item label="Application name">
              <el-input v-model="form.name" maxlength="80" />
            </el-form-item>
            <el-form-item label="Description">
              <el-input v-model="form.description" type="textarea" :rows="2" maxlength="500" />
            </el-form-item>
            <el-form-item label="Homepage URL">
              <el-input v-model="form.homepageUrl" placeholder="https://example.com" />
            </el-form-item>
            <el-form-item label="Redirect URIs">
              <el-input
                v-model="form.redirectUris"
                type="textarea"
                :rows="3"
                placeholder="https://example.com/callback"
              />
              <p class="dev-hint">
                One per line, up to {{ maxRedirectUris }}. HTTPS is required except for localhost.
              </p>
            </el-form-item>
          </el-form>

          <div class="dev-actions">
            <el-button type="primary" round :loading="saving" @click="save">Save changes</el-button>
          </div>
        </section>

        <section v-reveal="160" class="content-card dev-card danger">
          <div class="dev-card-head">
            <h2>Delete application</h2>
          </div>
          <p class="dev-hint">
            Deleting revokes every token issued to this application. This cannot be undone.
          </p>
          <div class="dev-actions">
            <el-button type="danger" plain round :loading="deleting" @click="remove">
              Delete application
            </el-button>
          </div>
        </section>
      </template>
    </div>

    <el-dialog
      v-model="secretOpen"
      title="New client secret"
      width="min(520px, 92vw)"
      align-center
      :close-on-click-modal="false"
    >
      <p class="dev-hint">
        Copy this secret now. It replaces the old secret immediately and will not be shown again.
      </p>
      <div class="dev-secret-row">
        <code>{{ newSecret }}</code>
        <el-button size="small" text @click="copy(newSecret)">Copy</el-button>
      </div>
      <template #footer>
        <el-button type="primary" round @click="secretOpen = false">I've saved it</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import LeftRail from '@/components/LeftRail.vue'
import { developerApi } from '@/api'
import { BASE_URL } from '@/api/client'
import type { DeveloperApp } from '@/types'
import { errorMessage } from '@/utils/format'

const route = useRoute()
const router = useRouter()

const app = ref<DeveloperApp | null>(null)
const loading = ref(true)
const error = ref('')
const saving = ref(false)
const deleting = ref(false)
const resetting = ref(false)
const maxRedirectUris = ref(5)

const secretOpen = ref(false)
const newSecret = ref('')

const form = reactive({
  name: '',
  description: '',
  homepageUrl: '',
  redirectUris: '',
  active: true,
})

const appBase = `${window.location.origin}${import.meta.env.BASE_URL.replace(/\/$/, '')}`

const endpoints = computed(() => ({
  authorize: `${appBase}/oauth/authorize`,
  token: `${window.location.origin}${BASE_URL.replace(/\/$/, '')}/oauth/token`,
  userinfo: `${window.location.origin}${BASE_URL.replace(/\/$/, '')}/oauth/userinfo`,
}))

const tokenExample = computed(() => {
  if (!app.value) return ''
  const redirect = app.value.redirectUris[0] || 'https://example.com/callback'
  if (app.value.clientType === 'PUBLIC') {
    return `curl -X POST '${endpoints.value.token}' \\
  -H 'Content-Type: application/x-www-form-urlencoded' \\
  -d 'grant_type=authorization_code' \\
  -d 'client_id=${app.value.clientId}' \\
  -d 'code=AUTHORIZATION_CODE' \\
  -d 'redirect_uri=${redirect}' \\
  -d 'code_verifier=YOUR_PKCE_VERIFIER'`
  }
  return `curl -X POST '${endpoints.value.token}' \\
  -u '${app.value.clientId}:YOUR_CLIENT_SECRET' \\
  -H 'Content-Type: application/x-www-form-urlencoded' \\
  -d 'grant_type=authorization_code' \\
  -d 'code=AUTHORIZATION_CODE' \\
  -d 'redirect_uri=${redirect}'`
})

function apply(data: DeveloperApp) {
  app.value = data
  form.name = data.name
  form.description = data.description || ''
  form.homepageUrl = data.homepageUrl || ''
  form.redirectUris = data.redirectUris.join('\n')
  form.active = data.status === 'ACTIVE'
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    const [data, list] = await Promise.all([
      developerApi.get(String(route.params.id)),
      developerApi.list().catch(() => null),
    ])
    apply(data)
    if (list) maxRedirectUris.value = list.maxRedirectUris ?? 5
  } catch (e) {
    error.value = errorMessage(e, 'Failed to load application')
  } finally {
    loading.value = false
  }
}

async function save() {
  if (!app.value) return
  const redirectUris = form.redirectUris
    .split(/\r?\n/)
    .map((value) => value.trim())
    .filter(Boolean)
  if (form.name.trim().length < 2) {
    ElMessage.warning('Application name must be at least 2 characters')
    return
  }
  if (!redirectUris.length) {
    ElMessage.warning('Add at least one redirect URI')
    return
  }
  saving.value = true
  try {
    const updated = await developerApi.update(app.value.id, {
      name: form.name.trim(),
      description: form.description.trim(),
      homepageUrl: form.homepageUrl.trim(),
      redirectUris,
      status: form.active ? 'ACTIVE' : 'DISABLED',
    })
    apply(updated)
    ElMessage.success('Saved')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to save'))
  } finally {
    saving.value = false
  }
}

async function resetSecret() {
  if (!app.value) return
  try {
    await ElMessageBox.confirm(
      'The current secret stops working immediately. Continue?',
      'Reset client secret',
      { confirmButtonText: 'Reset', cancelButtonText: 'Cancel', type: 'warning' }
    )
  } catch {
    return
  }
  resetting.value = true
  try {
    const updated = await developerApi.resetSecret(app.value.id)
    apply(updated)
    newSecret.value = updated.clientSecret || ''
    secretOpen.value = true
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to reset secret'))
  } finally {
    resetting.value = false
  }
}

async function remove() {
  if (!app.value) return
  try {
    await ElMessageBox.confirm(
      `Delete “${app.value.name}” and revoke all of its tokens? This cannot be undone.`,
      'Delete application',
      { confirmButtonText: 'Delete', cancelButtonText: 'Cancel', type: 'warning' }
    )
  } catch {
    return
  }
  deleting.value = true
  try {
    await developerApi.remove(app.value.id)
    ElMessage.success('Application deleted')
    router.push('/developers')
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to delete application'))
  } finally {
    deleting.value = false
  }
}

async function copy(value?: string | null) {
  if (!value) return
  try {
    await navigator.clipboard.writeText(value)
    ElMessage.success('Copied')
  } catch {
    ElMessage.warning('Copy failed — select the value manually')
  }
}

function copyEndpoints() {
  copy(
    `Authorize: ${endpoints.value.authorize}\nToken: ${endpoints.value.token}\nUserinfo: ${endpoints.value.userinfo}`
  )
}

onMounted(load)
</script>

<style scoped>
.dev-back {
  align-self: flex-start;
  margin-bottom: 10px;
  font-size: 12.5px;
  color: var(--of-text-muted);
  text-decoration: none;
  transition: color 0.3s var(--hp-ease);
}
.dev-back:hover {
  color: var(--of-brand);
}
.dev-detail-head {
  margin-bottom: 12px;
}
.dev-card {
  margin-bottom: 10px;
}
.dev-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
}
.dev-card-head h2 {
  margin: 0;
  font-size: 15px;
  font-weight: 650;
  letter-spacing: -0.01em;
  color: var(--of-text);
}
.dev-kv {
  display: grid;
  grid-template-columns: 120px minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  margin-bottom: 6px;
  border: 1px solid var(--hp-hairline);
  border-radius: 16px;
  background: var(--of-surface-weak);
}
.dev-kv-key {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--of-text-muted);
}
.dev-kv-value {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  color: var(--of-text);
  overflow-wrap: anywhere;
}
.dev-kv-value.plain {
  font-family: inherit;
  font-size: 12.5px;
}
.dev-badge {
  padding: 5px 10px;
  border-radius: 999px;
  border: 1px solid var(--hp-hairline);
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
.dev-badge.ok {
  color: #1d7a4c;
  border-color: rgba(29, 122, 76, 0.28);
  background: rgba(29, 122, 76, 0.08);
}
.dev-badge.off {
  color: var(--of-text-muted);
  background: var(--of-surface-weak);
}
.dev-redirects {
  margin-top: 12px;
}
.dev-redirects ul {
  margin: 6px 0 0;
  padding: 10px 14px 10px 28px;
  border: 1px dashed var(--hp-hairline);
  border-radius: 16px;
  color: var(--of-text-muted);
  font-size: 12px;
  line-height: 1.8;
}
.dev-redirects code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  color: var(--of-text);
  overflow-wrap: anywhere;
}
.dev-hint {
  margin: 8px 0 0;
  font-size: 11.5px;
  line-height: 1.6;
  color: var(--of-text-muted);
}
.dev-snippet {
  margin-top: 14px;
  border: 1px solid var(--hp-hairline);
  border-radius: 16px;
  background: var(--of-surface-weak);
  overflow: hidden;
}
.dev-snippet summary {
  padding: 10px 14px;
  cursor: pointer;
  font-size: 12.5px;
  color: var(--of-text-muted);
  list-style: none;
}
.dev-snippet summary::-webkit-details-marker {
  display: none;
}
.dev-snippet pre {
  margin: 0;
  padding: 12px 14px 14px;
  border-top: 1px solid var(--hp-hairline);
  overflow-x: auto;
}
.dev-snippet code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11.5px;
  line-height: 1.7;
  color: var(--of-text);
  white-space: pre;
}
.dev-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 16px;
}
.dev-card.danger {
  border-color: rgba(192, 38, 61, 0.22);
}
.dev-secret-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  border: 1px dashed var(--of-brand);
  border-radius: 16px;
  background: var(--of-surface-weak);
}
.dev-secret-row code {
  flex: 1;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  color: var(--of-text);
  overflow-wrap: anywhere;
}
@media (max-width: 640px) {
  .dev-kv {
    grid-template-columns: minmax(0, 1fr) auto;
  }
  .dev-kv-key {
    grid-column: 1 / -1;
  }
}
</style>
