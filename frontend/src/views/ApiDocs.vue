<template>
  <div class="page-grid no-right docs-page">
    <LeftRail />

    <div class="feed-column">
      <header v-reveal class="page-head docs-head">
        <span class="of-eyebrow">API reference</span>
        <h1 class="page-title">OpenForum API</h1>
        <p class="page-desc">
          Every endpoint exposed by the platform. Authenticate with a Bearer token from a normal
          login or from the OAuth 2.0 authorization code flow.
        </p>
        <div class="docs-head-actions">
          <el-button type="primary" round @click="router.push('/developers')">
            Create an application
          </el-button>
          <a :href="developerApi.specUrl()" target="_blank" rel="noopener">
            <el-button round>Download OpenAPI</el-button>
          </a>
        </div>
      </header>

      <div class="docs-layout">
        <nav class="docs-toc">
          <div class="rail-heading">On this page</div>
          <a href="#oauth" class="docs-toc-item">OAuth 2.0 guide</a>
          <a
            v-for="group in visibleGroups"
            :key="group.id"
            :href="`#group-${group.id}`"
            class="docs-toc-item"
          >
            {{ group.title }}
            <span>{{ group.endpoints.length }}</span>
          </a>
        </nav>

        <div class="docs-main">
          <section id="oauth" class="docs-section docs-oauth">
            <div class="docs-section-head">
              <h2>OAuth 2.0 guide</h2>
              <span class="docs-chip">Authorization code + PKCE</span>
            </div>
            <p class="docs-lead">
              Third-party applications never see a password. They receive an access token that acts
              as the authorizing user and inherits exactly that user's permissions.
            </p>

            <ol class="docs-steps">
              <li v-for="(step, i) in oauthSteps" :key="step.title">
                <span class="docs-step-no">{{ String(i + 1).padStart(2, '0') }}</span>
                <div>
                  <strong>{{ step.title }}</strong>
                  <p>{{ step.body }}</p>
                </div>
              </li>
            </ol>

            <div class="docs-code">
              <div class="docs-code-title">1 · Redirect the user</div>
              <pre><code>{{ oauthSamples.authorize }}</code></pre>
            </div>
            <div class="docs-code">
              <div class="docs-code-title">2 · Exchange the authorization code</div>
              <pre><code>{{ oauthSamples.token }}</code></pre>
            </div>
            <div class="docs-code">
              <div class="docs-code-title">3 · Call the API</div>
              <pre><code>{{ oauthSamples.userinfo }}</code></pre>
            </div>

            <div class="docs-callout">
              <strong>Refresh tokens</strong>
              <p>
                Access tokens expire after 2 hours. Refresh tokens last 30 days and rotate on every
                use — always store the newest one returned by <code>/oauth/token</code>.
              </p>
            </div>
          </section>

          <div class="docs-filter">
            <el-input
              v-model="filter"
              placeholder="Filter endpoints by path or description"
              clearable
            >
              <template #prefix>
                <IconSearch class="docs-filter-ico" />
              </template>
            </el-input>
            <el-button size="small" text @click="toggleAll">
              {{ allOpen ? 'Collapse all' : 'Expand all' }}
            </el-button>
          </div>

          <section
            v-for="group in visibleGroups"
            :id="`group-${group.id}`"
            :key="group.id"
            class="docs-section"
          >
            <div class="docs-section-head">
              <h2>{{ group.title }}</h2>
              <span class="docs-chip">
                {{ group.endpoints.length }}
                {{ group.endpoints.length === 1 ? 'endpoint' : 'endpoints' }}
              </span>
            </div>
            <p class="docs-lead">{{ group.description }}</p>

            <div class="docs-endpoints">
              <article
                v-for="endpoint in group.endpoints"
                :key="keyOf(group, endpoint)"
                class="docs-ep"
              >
                <button type="button" class="docs-ep-head" @click="toggle(group, endpoint)">
                  <span class="docs-method" :class="endpoint.method.toLowerCase()">
                    {{ endpoint.method }}
                  </span>
                  <code class="docs-path">{{ endpoint.path }}</code>
                  <span class="docs-ep-summary">{{ endpoint.summary }}</span>
                  <span class="docs-auth" :class="endpoint.auth">
                    {{ AUTH_LABELS[endpoint.auth] }}
                  </span>
                  <span class="docs-caret" :class="{ open: isOpen(group, endpoint) }">▾</span>
                </button>

                <div v-if="isOpen(group, endpoint)" class="docs-ep-body">
                  <p v-if="endpoint.notes" class="docs-note">{{ endpoint.notes }}</p>

                  <table v-if="endpoint.params?.length" class="docs-params">
                    <thead>
                      <tr>
                        <th>Name</th>
                        <th>In</th>
                        <th>Description</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="param in endpoint.params" :key="`${param.where}-${param.name}`">
                        <td><code>{{ param.name }}</code></td>
                        <td>{{ param.where }}</td>
                        <td>{{ param.description }}</td>
                      </tr>
                    </tbody>
                  </table>

                  <div v-if="endpoint.body" class="docs-code">
                    <div class="docs-code-title">Request body</div>
                    <pre><code>{{ endpoint.body }}</code></pre>
                  </div>
                  <div v-if="endpoint.response" class="docs-code">
                    <div class="docs-code-title">Example response</div>
                    <pre><code>{{ endpoint.response }}</code></pre>
                  </div>
                </div>
              </article>
            </div>
          </section>

          <div v-if="!visibleGroups.length" class="state-box">
            No endpoints match “{{ filter }}”.
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import LeftRail from '@/components/LeftRail.vue'
import IconSearch from '@/components/icons/IconSearch.vue'
import { developerApi } from '@/api'
import {
  apiGroups,
  AUTH_LABELS,
  oauthSamples,
  oauthSteps,
  type ApiEndpoint,
  type ApiGroup,
} from '@/data/apiReference'

const route = useRoute()
const router = useRouter()

const filter = ref('')
const open = reactive<Record<string, boolean>>({})

function keyOf(group: ApiGroup, endpoint: ApiEndpoint) {
  return `${group.id}:${endpoint.method}:${endpoint.path}`
}

function isOpen(group: ApiGroup, endpoint: ApiEndpoint) {
  return !!open[keyOf(group, endpoint)]
}

function toggle(group: ApiGroup, endpoint: ApiEndpoint) {
  const key = keyOf(group, endpoint)
  open[key] = !open[key]
}

const visibleGroups = computed<ApiGroup[]>(() => {
  const term = filter.value.trim().toLowerCase()
  if (!term) return apiGroups
  return apiGroups
    .map((group) => ({
      ...group,
      endpoints: group.endpoints.filter(
        (endpoint) =>
          endpoint.path.toLowerCase().includes(term) ||
          endpoint.summary.toLowerCase().includes(term) ||
          endpoint.method.toLowerCase() === term
      ),
    }))
    .filter((group) => group.endpoints.length > 0)
})

const allOpen = computed(() => {
  const endpoints = apiGroups.flatMap((group) =>
    group.endpoints.map((endpoint) => keyOf(group, endpoint))
  )
  return endpoints.length > 0 && endpoints.every((key) => open[key])
})

function toggleAll() {
  const next = !allOpen.value
  for (const group of apiGroups) {
    for (const endpoint of group.endpoints) {
      open[keyOf(group, endpoint)] = next
    }
  }
}

onMounted(async () => {
  if (!route.hash) return
  await nextTick()
  document.querySelector(route.hash)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
})
</script>

<style scoped>
.docs-head {
  margin-bottom: 10px;
}
.docs-head-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 16px;
}
.docs-head-actions a {
  text-decoration: none;
}
.docs-layout {
  display: grid;
  grid-template-columns: 190px minmax(0, 1fr);
  gap: 22px;
  align-items: start;
}
.docs-toc {
  position: sticky;
  top: 84px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 14px 10px;
  border: 1px solid var(--hp-hairline);
  border-radius: 24px;
  background: var(--hp-shell);
  box-shadow: var(--hp-soft);
}
.docs-toc .rail-heading {
  padding: 4px 10px 8px;
}
.docs-toc-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 7px 10px;
  border-radius: 12px;
  font-size: 12px;
  color: var(--of-text-muted);
  text-decoration: none;
  transition: background 0.3s var(--hp-ease), color 0.3s var(--hp-ease);
}
.docs-toc-item:hover {
  background: var(--of-surface-weak);
  color: var(--of-text);
}
.docs-toc-item span {
  font-size: 10.5px;
  color: var(--of-text-muted);
}
.docs-main {
  min-width: 0;
}
.docs-section {
  margin-bottom: 18px;
  padding: 24px 24px 20px;
  border: 1px solid var(--hp-hairline);
  border-radius: 30px;
  background: var(--hp-shell-strong);
  box-shadow: var(--hp-soft);
  scroll-margin-top: 90px;
}
.docs-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 10px;
  margin-bottom: 6px;
}
.docs-section-head h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--of-text);
}
.docs-chip {
  padding: 5px 10px;
  border: 1px solid var(--hp-hairline);
  border-radius: 999px;
  background: var(--of-surface-weak);
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--of-text-muted);
}
.docs-lead {
  margin: 0 0 16px;
  font-size: 13px;
  line-height: 1.7;
  color: var(--of-text-muted);
}
.docs-steps {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 0 0 18px;
  padding: 0;
  list-style: none;
}
.docs-steps li {
  display: flex;
  gap: 12px;
  padding: 12px 14px;
  border: 1px solid var(--hp-hairline);
  border-radius: 18px;
  background: var(--of-surface-weak);
}
.docs-step-no {
  color: var(--of-brand);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.12em;
  padding-top: 2px;
}
.docs-steps strong {
  display: block;
  font-size: 13px;
  color: var(--of-text);
}
.docs-steps p {
  margin: 3px 0 0;
  font-size: 12px;
  line-height: 1.65;
  color: var(--of-text-muted);
}
.docs-code {
  margin-bottom: 10px;
  border: 1px solid var(--hp-hairline);
  border-radius: 16px;
  background: var(--of-surface-weak);
  overflow: hidden;
}
.docs-code-title {
  padding: 8px 14px;
  border-bottom: 1px solid var(--hp-hairline);
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--of-text-muted);
}
.docs-code pre {
  margin: 0;
  padding: 12px 14px 14px;
  overflow-x: auto;
}
.docs-code code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11.5px;
  line-height: 1.7;
  color: var(--of-text);
  white-space: pre;
}
.docs-callout {
  margin-top: 12px;
  padding: 14px 16px;
  border: 1px dashed var(--of-brand);
  border-radius: 18px;
}
.docs-callout strong {
  font-size: 12.5px;
  color: var(--of-text);
}
.docs-callout p {
  margin: 4px 0 0;
  font-size: 12px;
  line-height: 1.7;
  color: var(--of-text-muted);
}
.docs-callout code,
.docs-note code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11.5px;
}
.docs-filter {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}
.docs-filter-ico {
  width: 14px;
  height: 14px;
}
.docs-endpoints {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.docs-ep {
  border: 1px solid var(--hp-hairline);
  border-radius: 16px;
  background: var(--of-surface-weak);
  overflow: hidden;
}
.docs-ep-head {
  display: grid;
  grid-template-columns: 62px minmax(0, 1.3fr) minmax(0, 1.1fr) auto 16px;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: none;
  text-align: left;
  cursor: pointer;
}
.docs-method {
  justify-self: start;
  padding: 3px 8px;
  border-radius: 8px;
  font-size: 9.5px;
  font-weight: 800;
  letter-spacing: 0.08em;
}
.docs-method.get {
  color: #1d6b4c;
  background: rgba(29, 122, 76, 0.12);
}
.docs-method.post {
  color: #92540b;
  background: rgba(200, 120, 20, 0.14);
}
.docs-method.put {
  color: #1f4c8c;
  background: rgba(38, 96, 180, 0.12);
}
.docs-method.delete {
  color: #a4111f;
  background: rgba(192, 38, 61, 0.12);
}
.docs-path {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  color: var(--of-text);
  overflow-wrap: anywhere;
}
.docs-ep-summary {
  font-size: 12px;
  color: var(--of-text-muted);
}
.docs-auth {
  padding: 3px 8px;
  border: 1px solid var(--hp-hairline);
  border-radius: 999px;
  font-size: 9.5px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--of-text-muted);
  white-space: nowrap;
}
.docs-auth.admin {
  color: #a4111f;
  border-color: rgba(192, 38, 61, 0.28);
}
.docs-auth.user {
  color: var(--of-text);
}
.docs-caret {
  font-size: 10px;
  color: var(--of-text-muted);
  transition: transform 0.3s var(--hp-ease);
}
.docs-caret.open {
  transform: rotate(180deg);
}
.docs-ep-body {
  padding: 0 14px 14px;
  border-top: 1px dashed var(--hp-hairline);
}
.docs-note {
  margin: 12px 0 10px;
  font-size: 12px;
  line-height: 1.7;
  color: var(--of-text-muted);
}
.docs-params {
  width: 100%;
  margin-top: 12px;
  border-collapse: collapse;
  font-size: 12px;
}
.docs-params th {
  padding: 8px 10px;
  border-bottom: 1px solid var(--hp-hairline);
  font-size: 9.5px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  text-align: left;
  color: var(--of-text-muted);
}
.docs-params td {
  padding: 8px 10px;
  border-bottom: 1px dashed var(--hp-hairline);
  color: var(--of-text-muted);
  vertical-align: top;
}
.docs-params td:first-child {
  white-space: nowrap;
}
.docs-params code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11.5px;
  color: var(--of-text);
}
.docs-ep-body .docs-code {
  margin-top: 10px;
  margin-bottom: 0;
}
@media (max-width: 1000px) {
  .docs-layout {
    grid-template-columns: minmax(0, 1fr);
  }
  .docs-toc {
    position: static;
    flex-direction: row;
    flex-wrap: wrap;
    gap: 6px;
    padding: 10px;
  }
  .docs-toc .rail-heading {
    display: none;
  }
  .docs-toc-item {
    border: 1px solid var(--hp-hairline);
  }
  .docs-toc-item span {
    display: none;
  }
}
@media (max-width: 720px) {
  .docs-section {
    padding: 18px 16px;
  }
  .docs-ep-head {
    grid-template-columns: 58px minmax(0, 1fr) 16px;
    row-gap: 4px;
  }
  .docs-ep-summary {
    grid-column: 2 / -1;
  }
  .docs-auth {
    display: none;
  }
  .docs-params td:first-child,
  .docs-params td:nth-child(2) {
    white-space: normal;
  }
}
</style>
