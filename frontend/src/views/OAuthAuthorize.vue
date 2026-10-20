<template>
  <div class="auth-page">
    <div v-reveal class="auth-panel oauth-panel">
      <el-skeleton v-if="loading" :rows="5" animated />

      <template v-else-if="error">
        <header class="auth-head">
          <span class="of-eyebrow">Authorization</span>
          <h1>This request can't be completed</h1>
          <p>{{ error }}</p>
        </header>
        <div class="oauth-actions">
          <el-button round @click="router.push('/developers/docs')">API reference</el-button>
          <el-button type="primary" round @click="router.push('/')">Back to OpenForum</el-button>
        </div>
      </template>

      <template v-else-if="info">
        <header class="auth-head oauth-head">
          <span class="of-eyebrow">Authorization request</span>
          <div class="oauth-app">
            <span class="oauth-app-mark">{{ initials(info.name) }}</span>
            <div>
              <h1>{{ info.name }}</h1>
              <p>
                wants to access your OpenForum account
                <template v-if="info.ownerUsername"> · by @{{ info.ownerUsername }}</template>
              </p>
            </div>
          </div>
        </header>

        <p v-if="info.description" class="oauth-desc">{{ info.description }}</p>

        <section class="oauth-scope">
          <div class="oauth-scope-head">
            <span class="oauth-scope-check">✓</span>
            <div>
              <strong>Act as your account</strong>
              <span class="oauth-scope-id">scope: {{ info.scope }}</span>
            </div>
          </div>
          <ul>
            <li>Read public communities, posts, and comments</li>
            <li>Post, comment, and vote on your behalf</li>
            <li>Manage your profile and homepages</li>
            <li>Call administrator APIs only if your account is an admin</li>
          </ul>
        </section>

        <dl class="oauth-meta">
          <div>
            <dt>Signed in as</dt>
            <dd>{{ auth.user?.username }}</dd>
          </div>
          <div>
            <dt>Redirects to</dt>
            <dd class="mono">{{ info.redirectUri }}</dd>
          </div>
          <div>
            <dt>Client ID</dt>
            <dd class="mono">{{ info.clientId }}</dd>
          </div>
        </dl>

        <p class="oauth-note">
          The application never receives your password. You can revoke access at any time by
          resetting your password or deleting the application in the developer center.
        </p>

        <div class="oauth-actions">
          <el-button round :disabled="submitting" @click="decide(false)">Cancel</el-button>
          <el-button type="primary" round :loading="submitting" @click="decide(true)">
            Authorize {{ info.name }}
          </el-button>
        </div>

        <button type="button" class="oauth-switch" @click="switchAccount">
          Not you? Sign in with another account
        </button>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { oauthApi } from '@/api'
import { useAuthStore } from '@/stores/auth'
import type { OAuthAuthorizeInfo } from '@/types'
import { errorMessage } from '@/utils/format'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const info = ref<OAuthAuthorizeInfo | null>(null)
const loading = ref(true)
const submitting = ref(false)
const error = ref('')

const clientId = String(route.query.client_id || '')
const redirectUri = String(route.query.redirect_uri || '')
const state = route.query.state ? String(route.query.state) : ''
const scope = route.query.scope ? String(route.query.scope) : ''
const codeChallenge = route.query.code_challenge ? String(route.query.code_challenge) : ''
const codeChallengeMethod = route.query.code_challenge_method
  ? String(route.query.code_challenge_method)
  : ''

function initials(name: string) {
  const parts = name.trim().split(/\s+/).filter(Boolean)
  if (!parts.length) return 'AP'
  return (parts[0][0] + (parts[1]?.[0] || '')).toUpperCase()
}

async function load() {
  if (!clientId) {
    error.value = 'Missing client_id. Start again from the application that sent you here.'
    loading.value = false
    return
  }
  try {
    info.value = await oauthApi.authorizeInfo({
      client_id: clientId,
      redirect_uri: redirectUri || undefined,
      scope: scope || undefined,
    })
  } catch (e) {
    error.value = errorMessage(e, 'Unknown or disabled application')
  } finally {
    loading.value = false
  }
}

async function decide(approve: boolean) {
  if (!info.value) return
  submitting.value = true
  try {
    const result = await oauthApi.authorize({
      clientId: info.value.clientId,
      redirectUri: info.value.redirectUri,
      state: state || undefined,
      codeChallenge: codeChallenge || undefined,
      codeChallengeMethod: codeChallengeMethod || undefined,
      scope: info.value.scope,
      approve,
    })
    window.location.href = result.redirectTo
  } catch (e) {
    ElMessage.error(errorMessage(e, approve ? 'Authorization failed' : 'Request failed'))
    submitting.value = false
  }
}

function switchAccount() {
  auth.logout()
  router.push({ name: 'login', query: { redirect: route.fullPath } })
}

onMounted(load)
</script>

<style scoped>
.oauth-panel {
  max-width: 560px;
}
.oauth-head {
  margin-bottom: 8px;
}
.oauth-app {
  display: flex;
  align-items: center;
  gap: 14px;
}
.oauth-app-mark {
  display: grid;
  place-items: center;
  width: 52px;
  height: 52px;
  flex: 0 0 52px;
  border-radius: 18px;
  background: var(--of-brand);
  color: #fff;
  font-size: 16px;
  font-weight: 700;
}
.oauth-app h1 {
  margin: 0 0 4px;
  font-size: 24px;
  font-weight: 700;
  letter-spacing: -0.03em;
  color: var(--of-text);
}
.oauth-app p {
  margin: 0;
  font-size: 12.5px;
  color: var(--of-text-muted);
}
.oauth-desc {
  margin: 0 0 16px;
  font-size: 13px;
  line-height: 1.7;
  color: var(--of-text-muted);
}
.oauth-scope {
  padding: 16px 18px;
  border: 1px solid var(--hp-hairline);
  border-radius: 22px;
  background: var(--of-surface-weak);
}
.oauth-scope-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}
.oauth-scope-check {
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  flex: 0 0 22px;
  border-radius: 999px;
  background: var(--of-brand);
  color: #fff;
  font-size: 12px;
  font-weight: 700;
}
.oauth-scope-head strong {
  display: block;
  font-size: 13px;
  color: var(--of-text);
}
.oauth-scope-id {
  font-size: 10.5px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  color: var(--of-text-muted);
}
.oauth-scope ul {
  margin: 0;
  padding-left: 34px;
  font-size: 12.5px;
  line-height: 1.9;
  color: var(--of-text-muted);
}
.oauth-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 16px 0 0;
}
.oauth-meta div {
  display: grid;
  grid-template-columns: 104px minmax(0, 1fr);
  gap: 10px;
  align-items: baseline;
}
.oauth-meta dt {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--of-text-muted);
}
.oauth-meta dd {
  margin: 0;
  font-size: 12.5px;
  color: var(--of-text);
  overflow-wrap: anywhere;
}
.mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11.5px;
}
.oauth-note {
  margin: 16px 0 0;
  font-size: 11.5px;
  line-height: 1.7;
  color: var(--of-text-muted);
}
.oauth-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 20px;
}
.oauth-actions .el-button {
  flex: 1;
  min-width: 140px;
  margin-left: 0;
}
.oauth-switch {
  margin-top: 14px;
  padding: 0;
  border: none;
  background: none;
  font-size: 11.5px;
  color: var(--of-text-muted);
  text-decoration: underline;
  cursor: pointer;
}
.oauth-switch:hover {
  color: var(--of-brand);
}
@media (max-width: 480px) {
  .oauth-meta div {
    grid-template-columns: minmax(0, 1fr);
    gap: 2px;
  }
}
</style>
