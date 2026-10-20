<template>
  <div class="page-grid">
    <LeftRail />

    <div class="feed-column">
      <header v-reveal class="page-head">
        <span class="of-eyebrow">Developer platform</span>
        <h1 class="page-title">Build on OpenForum</h1>
        <p class="page-desc">
          Register an OAuth application, let users sign in with their OpenForum account, and call
          every public API on their behalf.
        </p>
      </header>

      <section v-reveal="40" class="dev-steps">
        <article class="dev-step">
          <span class="dev-step-no">01</span>
          <h3>Create an application</h3>
          <p>Get a client ID and secret instantly — no review step.</p>
        </article>
        <article class="dev-step">
          <span class="dev-step-no">02</span>
          <h3>Authorize users</h3>
          <p>Send them to the consent screen with the OAuth 2.0 code flow.</p>
        </article>
        <article class="dev-step">
          <span class="dev-step-no">03</span>
          <h3>Call the API</h3>
          <p>Exchange the code for a Bearer token that carries their permissions.</p>
        </article>
      </section>

      <div v-if="!auth.isAuthenticated" v-reveal="80" class="state-box dev-guest">
        <p>Sign in to register applications and manage your credentials.</p>
        <div class="dev-guest-actions">
          <el-button type="primary" round @click="goLogin">Log in to continue</el-button>
          <el-button round text @click="router.push('/developers/docs')">
            Browse the API reference
          </el-button>
        </div>
      </div>

      <template v-else>
        <div v-reveal="80" class="dev-toolbar">
          <div class="dev-toolbar-meta">
            <strong>{{ apps.length }}</strong>
            <span>/ {{ maxApps }} applications</span>
            <span v-if="loaded" class="dev-toolbar-dot">·</span>
            <span v-if="loaded">{{ maxRedirectUris }} redirect URIs each</span>
          </div>
          <el-button
            type="primary"
            round
            :disabled="apps.length >= maxApps"
            @click="openCreate"
          >
            <IconPlus />
            <span>New application</span>
          </el-button>
        </div>

        <el-skeleton v-if="loading" :rows="5" animated />

        <div v-else-if="error" class="state-box error">
          {{ error }}
          <div style="margin-top: 12px">
            <el-button size="small" @click="load">Retry</el-button>
          </div>
        </div>

        <div v-else-if="!apps.length" class="state-box">
          <p>You haven't registered any applications yet.</p>
          <div style="margin-top: 12px">
            <el-button type="primary" round @click="openCreate">Create your first app</el-button>
          </div>
        </div>

        <div v-else class="dev-list">
          <article
            v-for="(app, i) in apps"
            :key="app.id"
            v-reveal="Math.min(i, 6) * 45"
            class="dev-app"
            role="button"
            tabindex="0"
            @click="open(app)"
            @keyup.enter="open(app)"
          >
            <div class="dev-app-top">
              <span class="dev-app-mark" :class="{ public: app.clientType === 'PUBLIC' }">
                {{ initials(app.name) }}
              </span>
              <div class="dev-app-title">
                <h3>{{ app.name }}</h3>
                <p>{{ app.description || 'No description yet' }}</p>
              </div>
              <span class="dev-badge" :class="app.status === 'ACTIVE' ? 'ok' : 'off'">
                {{ app.status === 'ACTIVE' ? 'Active' : 'Disabled' }}
              </span>
            </div>

            <dl class="dev-app-meta">
              <div>
                <dt>Client ID</dt>
                <dd class="mono">{{ app.clientId }}</dd>
              </div>
              <div>
                <dt>Type</dt>
                <dd>{{ app.clientType === 'PUBLIC' ? 'Public (PKCE)' : 'Confidential' }}</dd>
              </div>
              <div>
                <dt>Redirects</dt>
                <dd>{{ app.redirectUris.length }}</dd>
              </div>
              <div>
                <dt>Created</dt>
                <dd>{{ formatDate(app.createdAt) }}</dd>
              </div>
            </dl>
          </article>
        </div>
      </template>
    </div>

    <aside class="rail-right">
      <section v-reveal="60" class="rail-card dev-resources">
        <div class="rail-heading">Developer resources</div>
        <router-link to="/developers/docs" class="dev-resource">
          <span class="dev-resource-title">API reference</span>
          <span class="dev-resource-desc">Every endpoint, parameter, and response shape.</span>
        </router-link>
        <router-link to="/developers/docs#oauth" class="dev-resource">
          <span class="dev-resource-title">OAuth 2.0 guide</span>
          <span class="dev-resource-desc">Authorization code flow with PKCE, step by step.</span>
        </router-link>
        <a :href="developerApi.specUrl()" class="dev-resource" target="_blank" rel="noopener">
          <span class="dev-resource-title">OpenAPI document</span>
          <span class="dev-resource-desc">Download the machine-readable spec.</span>
        </a>
      </section>

      <section v-reveal="120" class="rail-card dev-note">
        <div class="rail-heading">Good to know</div>
        <p>
          Tokens act as the authorizing user, so an app can never do more than the person who
          approved it.
        </p>
      </section>
    </aside>

    <el-dialog
      v-model="createOpen"
      title="New application"
      width="min(560px, 92vw)"
      align-center
      destroy-on-close
    >
      <el-form label-position="top" class="dev-form">
        <el-form-item label="Application name">
          <el-input v-model="form.name" maxlength="80" placeholder="e.g. Forum Digest" />
        </el-form-item>
        <el-form-item label="Description">
          <el-input
            v-model="form.description"
            type="textarea"
            :rows="2"
            maxlength="500"
            placeholder="What does your application do?"
          />
        </el-form-item>
        <el-form-item label="Homepage URL (optional)">
          <el-input v-model="form.homepageUrl" placeholder="https://example.com" />
        </el-form-item>
        <el-form-item label="Application type">
          <div class="dev-type-grid">
            <button
              v-for="option in typeOptions"
              :key="option.value"
              type="button"
              class="dev-type"
              :class="{ active: form.clientType === option.value }"
              @click="form.clientType = option.value"
            >
              <strong>{{ option.label }}</strong>
              <span>{{ option.hint }}</span>
            </button>
          </div>
        </el-form-item>
        <el-form-item label="Redirect URIs">
          <el-input
            v-model="form.redirectUris"
            type="textarea"
            :rows="3"
            placeholder="https://example.com/callback&#10;http://localhost:3000/callback"
          />
          <p class="dev-hint">
            One per line, up to {{ maxRedirectUris }}. HTTPS is required except for localhost.
          </p>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button round @click="createOpen = false">Cancel</el-button>
        <el-button type="primary" round :loading="creating" @click="submitCreate">
          Create application
        </el-button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="secretOpen"
      title="Your application credentials"
      width="min(560px, 92vw)"
      align-center
      :close-on-click-modal="false"
    >
      <p class="dev-secret-lead">
        Copy the client secret now — it is stored hashed and will never be shown again.
      </p>
      <div class="dev-secret-row">
        <span class="dev-secret-label">Client ID</span>
        <code>{{ created?.clientId }}</code>
        <el-button size="small" text @click="copy(created?.clientId)">Copy</el-button>
      </div>
      <div v-if="created?.clientSecret" class="dev-secret-row">
        <span class="dev-secret-label">Client secret</span>
        <code>{{ created.clientSecret }}</code>
        <el-button size="small" text @click="copy(created?.clientSecret)">Copy</el-button>
      </div>
      <p v-else class="dev-hint">
        This is a public client: it authenticates with PKCE and has no client secret.
      </p>
      <template #footer>
        <el-button type="primary" round @click="finishCreated">I've saved it</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import LeftRail from '@/components/LeftRail.vue'
import IconPlus from '@/components/icons/IconPlus.vue'
import { developerApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { DeveloperApp, OAuthClientType } from '@/types'
import { errorMessage, formatDate } from '@/utils/format'

const auth = useAuthStore()
const router = useRouter()

const apps = ref<DeveloperApp[]>([])
const loading = ref(false)
const loaded = ref(false)
const error = ref('')
const maxApps = ref(10)
const maxRedirectUris = ref(5)

const createOpen = ref(false)
const creating = ref(false)
const secretOpen = ref(false)
const created = ref<DeveloperApp | null>(null)

const form = reactive({
  name: '',
  description: '',
  homepageUrl: '',
  redirectUris: '',
  clientType: 'CONFIDENTIAL' as OAuthClientType,
})

const typeOptions = [
  {
    value: 'CONFIDENTIAL' as OAuthClientType,
    label: 'Confidential',
    hint: 'Server-side app with a client secret',
  },
  {
    value: 'PUBLIC' as OAuthClientType,
    label: 'Public',
    hint: 'SPA or mobile app using PKCE',
  },
]

function initials(name: string) {
  const parts = name.trim().split(/\s+/).filter(Boolean)
  if (!parts.length) return 'AP'
  return (parts[0][0] + (parts[1]?.[0] || '')).toUpperCase()
}

function goLogin() {
  router.push({ name: 'login', query: { redirect: '/developers' } })
}

function open(app: DeveloperApp) {
  router.push(`/developers/apps/${app.id}`)
}

async function load() {
  if (!auth.isAuthenticated) {
    loaded.value = true
    return
  }
  loading.value = true
  error.value = ''
  try {
    const data = await developerApi.list()
    apps.value = data.content || []
    maxApps.value = data.maxApps ?? 10
    maxRedirectUris.value = data.maxRedirectUris ?? 5
  } catch (e) {
    error.value = errorMessage(e, 'Failed to load applications')
  } finally {
    loading.value = false
    loaded.value = true
  }
}

function openCreate() {
  form.name = ''
  form.description = ''
  form.homepageUrl = ''
  form.redirectUris = ''
  form.clientType = 'CONFIDENTIAL'
  createOpen.value = true
}

async function submitCreate() {
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
  creating.value = true
  try {
    const app = await developerApi.create({
      name: form.name.trim(),
      description: form.description.trim() || undefined,
      homepageUrl: form.homepageUrl.trim() || undefined,
      redirectUris,
      clientType: form.clientType,
    })
    createOpen.value = false
    created.value = app
    secretOpen.value = true
    await load()
  } catch (e) {
    ElMessage.error(errorMessage(e, 'Failed to create application'))
  } finally {
    creating.value = false
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

function finishCreated() {
  secretOpen.value = false
  if (created.value) router.push(`/developers/apps/${created.value.id}`)
}

onMounted(load)
</script>

<style scoped>
.dev-steps {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
  margin-bottom: 6px;
}
.dev-step {
  padding: 20px 22px 22px;
  border: 1px solid var(--hp-hairline);
  border-radius: 26px;
  background: var(--hp-shell);
  box-shadow: var(--hp-soft);
}
.dev-step-no {
  display: inline-block;
  margin-bottom: 14px;
  color: var(--of-brand);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
}
.dev-step h3 {
  margin: 0 0 6px;
  font-size: 15px;
  font-weight: 650;
  letter-spacing: -0.01em;
  color: var(--of-text);
}
.dev-step p {
  margin: 0;
  font-size: 12.5px;
  line-height: 1.6;
  color: var(--of-text-muted);
}
.dev-guest-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 10px;
}
.dev-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 10px 12px 10px 20px;
  border: 1px solid var(--hp-hairline);
  border-radius: 999px;
  background: var(--hp-shell);
  box-shadow: var(--hp-soft);
}
.dev-toolbar-meta {
  display: flex;
  align-items: baseline;
  gap: 6px;
  font-size: 13px;
  color: var(--of-text-muted);
}
.dev-toolbar-meta strong {
  color: var(--of-text);
  font-size: 15px;
}
.dev-toolbar-dot {
  color: var(--of-border-strong);
}
.dev-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.dev-app {
  padding: 20px 22px;
  border: 1px solid var(--hp-hairline);
  border-radius: 28px;
  background: var(--hp-shell);
  box-shadow: var(--hp-soft);
  cursor: pointer;
  transition: transform 0.5s var(--hp-ease), box-shadow 0.5s var(--hp-ease),
    border-color 0.5s var(--hp-ease);
}
.dev-app:hover {
  transform: translateY(-2px);
  border-color: var(--of-brand);
  box-shadow: var(--hp-glow);
}
.dev-app-top {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}
.dev-app-mark {
  display: grid;
  place-items: center;
  width: 44px;
  height: 44px;
  flex: 0 0 44px;
  border-radius: 16px;
  background: var(--of-brand);
  color: #fff;
  font-size: 14px;
  font-weight: 700;
  letter-spacing: 0.02em;
}
.dev-app-mark.public {
  background: var(--of-text);
}
.dev-app-title {
  flex: 1;
  min-width: 0;
}
.dev-app-title h3 {
  margin: 0 0 3px;
  font-size: 16px;
  font-weight: 650;
  letter-spacing: -0.015em;
  color: var(--of-text);
}
.dev-app-title p {
  margin: 0;
  font-size: 12.5px;
  color: var(--of-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
.dev-app-meta {
  display: grid;
  grid-template-columns: minmax(0, 1.6fr) repeat(3, minmax(0, 0.8fr));
  gap: 12px;
  margin: 16px 0 0;
  padding-top: 14px;
  border-top: 1px dashed var(--hp-hairline);
}
.dev-app-meta dt {
  margin-bottom: 4px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--of-text-muted);
}
.dev-app-meta dd {
  margin: 0;
  font-size: 12.5px;
  color: var(--of-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}
.dev-resource {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 12px 14px;
  border-radius: 18px;
  color: inherit;
  text-decoration: none;
  transition: background 0.4s var(--hp-ease);
}
.dev-resource:hover {
  background: var(--of-surface-weak);
}
.dev-resource-title {
  font-size: 13px;
  font-weight: 650;
  color: var(--of-text);
}
.dev-resource-desc {
  font-size: 11.5px;
  line-height: 1.5;
  color: var(--of-text-muted);
}
.dev-note p {
  margin: 6px 4px 4px;
  font-size: 12px;
  line-height: 1.7;
  color: var(--of-text-muted);
}
.dev-type-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  width: 100%;
}
.dev-type {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 12px 14px;
  border: 1px solid var(--hp-hairline);
  border-radius: 18px;
  background: var(--of-surface-weak);
  text-align: left;
  cursor: pointer;
  transition: border-color 0.4s var(--hp-ease), background 0.4s var(--hp-ease);
}
.dev-type strong {
  font-size: 13px;
  color: var(--of-text);
}
.dev-type span {
  font-size: 11.5px;
  color: var(--of-text-muted);
}
.dev-type.active {
  border-color: var(--of-brand);
  background: var(--hp-shell-strong);
}
.dev-hint {
  margin: 6px 0 0;
  font-size: 11.5px;
  line-height: 1.6;
  color: var(--of-text-muted);
}
.dev-secret-lead {
  margin: 0 0 16px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--of-text-muted);
}
.dev-secret-row {
  display: grid;
  grid-template-columns: 108px minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  margin-bottom: 8px;
  border: 1px solid var(--hp-hairline);
  border-radius: 16px;
  background: var(--of-surface-weak);
}
.dev-secret-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--of-text-muted);
}
.dev-secret-row code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  color: var(--of-text);
  word-break: break-all;
}
@media (max-width: 900px) {
  .dev-steps {
    grid-template-columns: minmax(0, 1fr);
  }
}
@media (max-width: 640px) {
  .dev-toolbar {
    flex-direction: column;
    align-items: stretch;
    border-radius: 24px;
    padding: 14px;
  }
  .dev-app-meta {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .dev-secret-row {
    grid-template-columns: minmax(0, 1fr) auto;
  }
  .dev-secret-row .dev-secret-label {
    grid-column: 1 / -1;
  }
  .dev-type-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
